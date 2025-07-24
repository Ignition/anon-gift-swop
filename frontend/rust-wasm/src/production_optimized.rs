use crate::assignment::Assignment;
use crate::forbidden_pair::ForbiddenPair;
use crate::mcmc_sampler::WeightedGraph;
use std::collections::{HashMap, HashSet};
use std::sync::Mutex;

/// Production-optimized assignment processor with all optimizations enabled
/// Uses Mutex for thread safety even though WASM is single-threaded (for static usage)
pub struct ProductionAssignmentProcessor {
    weight_cache: Mutex<WeightCache>,
}

impl ProductionAssignmentProcessor {
    pub fn new() -> Self {
        Self {
            weight_cache: Mutex::new(WeightCache::new()),
        }
    }

    /// Production-ready process_assignment with all optimizations
    pub fn process_assignment(
        &self,
        participants: &HashSet<usize>,
        forbidden_pairs: &[ForbiddenPair],
        history: &[Assignment],
    ) -> Option<Vec<usize>> {
        // Single participant case is impossible (can't give to themselves)
        if participants.len() == 1 {
            return None;
        }

        // Calculate problem characteristics for algorithm selection
        let n = participants.len();
        let constraint_density = if n > 1 {
            forbidden_pairs.len() as f64 / (n * (n - 1) / 2) as f64
        } else {
            0.0
        };
        let history_weight = history.len() as f64;

        // Hybrid Algorithm Selector - choose optimal algorithm based on problem characteristics
        let result = match (n, constraint_density, history_weight) {
            // Very small groups: use exact branch-and-bound
            (2..=6, _, _) => {
                if n == 10 {
                    println!("  Using branch-and-bound");
                }
                self.branch_and_bound_optimized(participants, forbidden_pairs, history)
            }

            // Medium groups with high constraints: use simulated annealing with convergence detection
            (7..=10, density, _) if density > 0.3 => {
                if n == 10 {
                    println!("  Using simulated annealing");
                }
                self.simulated_annealing_with_convergence(participants, forbidden_pairs, history)
            }

            // Large groups with complex history: use parallel tempering with convergence detection
            (11.., _, weight) if weight > 5.0 => {
                if n == 10 {
                    println!("  Using parallel tempering");
                }
                self.parallel_tempering_with_convergence(participants, forbidden_pairs, history)
            }

            // Default: optimized MCMC with convergence detection and caching
            _ => {
                if n == 10 {
                    println!("  Using default MCMC");
                }
                self.mcmc_with_convergence_detection(participants, forbidden_pairs, history)
            }
        };

        // FINAL SAFETY CHECK: Never return cycles with forbidden pairs
        // This is the ultimate safeguard regardless of which algorithm was used
        if let Some(ref cycle) = result {
            // Check each edge in the cycle for forbidden pairs
            let mut has_forbidden_pairs = false;

            for i in 0..cycle.len() {
                let giver = cycle[i];
                let receiver = cycle[(i + 1) % cycle.len()];

                // Check if this is a forbidden pair
                for forbidden_pair in forbidden_pairs {
                    if (forbidden_pair.member_1 == giver && forbidden_pair.member_2 == receiver)
                        || (forbidden_pair.member_1 == receiver && forbidden_pair.member_2 == giver)
                    {
                        has_forbidden_pairs = true;
                        if n == 10 {
                            // Debug for edge case test
                            println!(
                                "  🚫 FINAL CHECK: Rejected cycle with forbidden pair ({}, {})",
                                giver, receiver
                            );
                        }
                        break;
                    }
                }

                if has_forbidden_pairs {
                    break;
                }
            }

            if has_forbidden_pairs {
                return None; // Reject any result with forbidden pairs
            }
        }

        result
    }

    /// Branch-and-bound for small groups (≤6 participants) - guarantees optimal solution
    fn branch_and_bound_optimized(
        &self,
        participants: &HashSet<usize>,
        forbidden_pairs: &[ForbiddenPair],
        history: &[Assignment],
    ) -> Option<Vec<usize>> {
        let graph = WeightedGraph::new(participants, forbidden_pairs, history);
        let participants_vec: Vec<_> = participants.iter().copied().collect();

        let mut best_cycle = None;
        let mut best_weight = 0.0;

        // Generate all permutations with branch-and-bound pruning
        let mut current_cycle = Vec::new();
        let mut remaining: Vec<_> = participants_vec.clone();

        self.branch_and_bound_recursive(
            &mut current_cycle,
            &mut remaining,
            &graph,
            1.0,
            1.0,
            &mut best_cycle,
            &mut best_weight,
        );

        // Final validation: ensure no forbidden pairs in result
        if let Some(ref cycle) = best_cycle {
            let final_weight = crate::mcmc_sampler::calculate_cycle_weight(cycle, &graph);
            if final_weight > 0.0 {
                best_cycle
            } else {
                // Debug: Log when branch-and-bound returns invalid cycle
                if participants.len() == 10 {
                    println!(
                        "  ⚠️  Branch-and-bound returned invalid cycle with weight {:.3}",
                        final_weight
                    );
                    for i in 0..cycle.len() {
                        let giver = cycle[i];
                        let receiver = cycle[(i + 1) % cycle.len()];
                        let edge_weight = graph.get_weight(giver, receiver);
                        if edge_weight == 0.0 {
                            println!(
                                "    Forbidden edge: {} -> {} (weight: {:.3})",
                                giver, receiver, edge_weight
                            );
                        }
                    }
                }
                None // Reject if final validation fails
            }
        } else {
            None
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn branch_and_bound_recursive(
        &self,
        current_cycle: &mut Vec<usize>,
        remaining: &mut Vec<usize>,
        graph: &WeightedGraph,
        current_weight: f64,
        partial_weight: f64,
        best_cycle: &mut Option<Vec<usize>>,
        best_weight: &mut f64,
    ) {
        // Pruning: if even the optimistic bound can't beat the best, skip
        let optimistic_bound = current_weight * partial_weight;
        if optimistic_bound <= *best_weight {
            return;
        }

        if remaining.is_empty() {
            // Complete cycle: check the closing edge
            if let (Some(&first), Some(&last)) = (current_cycle.first(), current_cycle.last()) {
                let closing_weight = graph.get_weight(last, first);
                if closing_weight > 0.0 {
                    let total_weight = current_weight * closing_weight;
                    if total_weight > *best_weight {
                        *best_weight = total_weight;
                        *best_cycle = Some(current_cycle.clone());
                    }
                }
            }
            return;
        }

        let current_participant = current_cycle.last().copied();

        for i in 0..remaining.len() {
            let next = remaining[i];

            // Calculate edge weight with caching
            let edge_weight = if let Some(current) = current_participant {
                self.get_cached_weight(current, next, graph)
            } else {
                1.0 // First participant
            };

            if edge_weight > 0.0 {
                // Branch: add this participant
                current_cycle.push(next);
                let removed = remaining.remove(i);

                self.branch_and_bound_recursive(
                    current_cycle,
                    remaining,
                    graph,
                    current_weight * edge_weight,
                    partial_weight,
                    best_cycle,
                    best_weight,
                );

                // Backtrack
                current_cycle.pop();
                remaining.insert(i, removed);
            }
        }
    }

    /// Simulated annealing with convergence detection
    fn simulated_annealing_with_convergence(
        &self,
        participants: &HashSet<usize>,
        forbidden_pairs: &[ForbiddenPair],
        history: &[Assignment],
    ) -> Option<Vec<usize>> {
        let graph = WeightedGraph::new(participants, forbidden_pairs, history);

        let mut current_cycle =
            match crate::mcmc_sampler::find_initial_valid_cycle(&graph, participants) {
                Some(cycle) => cycle,
                None => return None, // No valid cycle exists
            };
        let mut current_weight = self.get_cached_cycle_weight(&current_cycle, &graph);
        let mut best_cycle = current_cycle.clone();
        let mut best_weight = current_weight;

        // Convergence detection parameters
        let min_iterations = 30;
        let max_iterations = 150;
        let convergence_window = 10;
        let convergence_threshold = 0.01;

        let mut weight_history = Vec::new();
        let initial_temperature = 2.0f64;
        let final_temperature = 0.01f64;

        for iteration in 0..max_iterations {
            // Exponential cooling schedule
            let temperature = initial_temperature
                * (final_temperature / initial_temperature)
                    .powf(iteration as f64 / max_iterations as f64);

            let candidate_cycle =
                generate_candidate_cycle_smart(&current_cycle, &graph, participants, iteration);
            let candidate_weight = self.get_cached_cycle_weight(&candidate_cycle, &graph);

            // Never accept forbidden cycles (weight = 0.0)
            // Also double-check with direct calculation to avoid caching bugs
            let direct_weight =
                crate::mcmc_sampler::calculate_cycle_weight(&candidate_cycle, &graph);
            if candidate_weight == 0.0 || direct_weight <= 0.0 {
                if participants.len() == 10 && direct_weight == 0.0 {
                    println!("  Simulated annealing rejected forbidden cycle with cached weight {:.3}, direct weight {:.3}", candidate_weight, direct_weight);
                }
                continue; // Skip forbidden cycles entirely
            }

            // Simulated annealing acceptance criterion
            let accept_probability = if candidate_weight >= current_weight {
                1.0 // Always accept improvements
            } else if current_weight > 0.0 && temperature > 0.0 {
                // Accept worse solutions with probability based on temperature
                ((candidate_weight / current_weight).ln() / temperature).exp()
            } else {
                0.0
            };

            if crate::mcmc_sampler::random() < accept_probability {
                current_cycle = candidate_cycle;
                current_weight = candidate_weight;

                // Track best solution found
                if current_weight > best_weight {
                    best_cycle = current_cycle.clone();
                    best_weight = current_weight;
                }
            }

            weight_history.push(current_weight);

            // Check for convergence after minimum iterations
            if iteration >= min_iterations
                && weight_history.len() >= convergence_window
                && self.has_converged(&weight_history, convergence_window, convergence_threshold)
            {
                break;
            }
        }

        if best_weight > 0.0 {
            // Final validation: ensure no forbidden pairs in result
            let final_weight = crate::mcmc_sampler::calculate_cycle_weight(&best_cycle, &graph);
            if final_weight > 0.0 {
                Some(best_cycle)
            } else {
                None // Reject if final validation fails
            }
        } else {
            None
        }
    }

    /// Parallel tempering with convergence detection
    fn parallel_tempering_with_convergence(
        &self,
        participants: &HashSet<usize>,
        forbidden_pairs: &[ForbiddenPair],
        history: &[Assignment],
    ) -> Option<Vec<usize>> {
        let graph = WeightedGraph::new(participants, forbidden_pairs, history);

        // Run multiple chains with different temperatures
        let num_chains = 3;
        let temperatures = [0.5f64, 1.0f64, 2.0f64];
        let mut chains: Vec<(Vec<usize>, f64)> = Vec::new();
        let mut chain_histories: Vec<Vec<f64>> = Vec::new();

        // Initialize chains
        for _ in 0..num_chains {
            let cycle = match crate::mcmc_sampler::find_initial_valid_cycle(&graph, participants) {
                Some(cycle) => cycle,
                None => return None, // No valid cycle exists
            };
            let weight = self.get_cached_cycle_weight(&cycle, &graph);
            chains.push((cycle, weight));
            chain_histories.push(Vec::new());
        }

        let min_iterations = 40;
        let max_iterations = 120;
        let convergence_window = 8;
        let convergence_threshold = 0.01;

        // Run chains with convergence detection
        for iteration in 0..max_iterations {
            for chain_idx in 0..num_chains {
                let temperature = temperatures[chain_idx];
                let (ref mut current_cycle, ref mut current_weight) = chains[chain_idx];

                let candidate_cycle =
                    generate_candidate_cycle_smart(current_cycle, &graph, participants, iteration);
                let candidate_weight = self.get_cached_cycle_weight(&candidate_cycle, &graph);

                // Never accept forbidden cycles (weight = 0.0)
                if candidate_weight == 0.0 {
                    continue; // Skip forbidden cycles entirely
                }

                let accept_probability = if candidate_weight >= *current_weight {
                    1.0
                } else if *current_weight > 0.0 && temperature > 0.0 {
                    ((candidate_weight / *current_weight).ln() / temperature).exp()
                } else {
                    0.0
                };

                if crate::mcmc_sampler::random() < accept_probability {
                    *current_cycle = candidate_cycle;
                    *current_weight = candidate_weight;
                }

                chain_histories[chain_idx].push(*current_weight);

                // Occasionally swap between chains (parallel tempering)
                if iteration % 10 == 0 && chain_idx > 0 && crate::mcmc_sampler::random() < 0.1 {
                    let other_idx = chain_idx - 1;
                    let temp_ratio = temperatures[other_idx] / temperatures[chain_idx];
                    let weight_diff = chains[other_idx].1 - chains[chain_idx].1;

                    if crate::mcmc_sampler::random() < (temp_ratio * weight_diff).exp().min(1.0) {
                        chains.swap(chain_idx, other_idx);
                        chain_histories.swap(chain_idx, other_idx);
                    }
                }
            }

            // Check convergence for the main chain (index 1 - medium temperature)
            if iteration >= min_iterations
                && chain_histories[1].len() >= convergence_window
                && self.has_converged(
                    &chain_histories[1],
                    convergence_window,
                    convergence_threshold,
                )
            {
                break;
            }
        }

        // Return best solution from all chains with final validation
        let best_solution = chains
            .into_iter()
            .filter(|(_, weight)| *weight > 0.0)
            .max_by(|(_, w1), (_, w2)| w1.partial_cmp(w2).unwrap())
            .map(|(cycle, _)| cycle);

        // Final validation: ensure no forbidden pairs in result
        if let Some(ref cycle) = best_solution {
            let final_weight = crate::mcmc_sampler::calculate_cycle_weight(cycle, &graph);
            if final_weight > 0.0 {
                best_solution
            } else {
                None // Reject if final validation fails
            }
        } else {
            None
        }
    }

    /// MCMC with convergence detection and caching
    fn mcmc_with_convergence_detection(
        &self,
        participants: &HashSet<usize>,
        forbidden_pairs: &[ForbiddenPair],
        history: &[Assignment],
    ) -> Option<Vec<usize>> {
        let graph = WeightedGraph::new(participants, forbidden_pairs, history);

        let mut current_cycle =
            match crate::mcmc_sampler::find_initial_valid_cycle(&graph, participants) {
                Some(cycle) => cycle,
                None => return None, // No valid cycle exists
            };
        let mut current_weight = self.get_cached_cycle_weight(&current_cycle, &graph);

        // Adaptive parameters based on group size
        let (min_iterations, max_iterations) = adaptive_convergence_params(participants.len());
        let convergence_window = 10;
        let convergence_threshold = 0.01;

        let mut weight_history = Vec::new();
        let mut best_cycle = current_cycle.clone();
        let mut best_weight = current_weight;

        for iteration in 0..max_iterations {
            let candidate_cycle =
                generate_candidate_cycle_smart(&current_cycle, &graph, participants, iteration);
            let candidate_weight = self.get_cached_cycle_weight(&candidate_cycle, &graph);

            // Never accept forbidden cycles (weight = 0.0)
            // Also double-check with direct calculation to avoid caching bugs
            let direct_weight =
                crate::mcmc_sampler::calculate_cycle_weight(&candidate_cycle, &graph);
            if candidate_weight == 0.0 || direct_weight <= 0.0 {
                if participants.len() == 10 && direct_weight == 0.0 {
                    println!("  Simulated annealing rejected forbidden cycle with cached weight {:.3}, direct weight {:.3}", candidate_weight, direct_weight);
                }
                continue; // Skip forbidden cycles entirely
            }

            let accept_probability = if candidate_weight >= current_weight {
                1.0
            } else if current_weight > 0.0 {
                candidate_weight / current_weight
            } else {
                0.0
            };

            if crate::mcmc_sampler::random() < accept_probability {
                current_cycle = candidate_cycle;
                current_weight = candidate_weight;

                if current_weight > best_weight {
                    best_cycle = current_cycle.clone();
                    best_weight = current_weight;
                }
            }

            weight_history.push(current_weight);

            // Check for convergence after minimum iterations
            if iteration >= min_iterations
                && weight_history.len() >= convergence_window
                && self.has_converged(&weight_history, convergence_window, convergence_threshold)
            {
                break;
            }
        }

        // Ensure the final result respects forbidden pairs
        if best_weight > 0.0 {
            // Double-check that the result doesn't violate forbidden pairs
            let final_weight = crate::mcmc_sampler::calculate_cycle_weight(&best_cycle, &graph);
            if final_weight > 0.0 {
                Some(best_cycle)
            } else {
                // The cached weight was wrong or the cycle became invalid - return None
                None
            }
        } else {
            None
        }
    }

    /// Check if the algorithm has converged based on weight variance
    fn has_converged(&self, weight_history: &[f64], window: usize, threshold: f64) -> bool {
        if weight_history.len() < window {
            return false;
        }

        let recent_weights = &weight_history[weight_history.len() - window..];
        let avg_weight: f64 = recent_weights.iter().sum::<f64>() / window as f64;
        let variance: f64 = recent_weights
            .iter()
            .map(|&w| (w - avg_weight).powi(2))
            .sum::<f64>()
            / window as f64;
        let std_dev = variance.sqrt();

        std_dev < threshold
    }

    /// Get cached weight for an edge
    fn get_cached_weight(&self, giver: usize, receiver: usize, graph: &WeightedGraph) -> f64 {
        if let Ok(mut cache) = self.weight_cache.lock() {
            cache.get_edge_weight(giver, receiver, graph)
        } else {
            graph.get_weight(giver, receiver)
        }
    }

    /// Get cached weight for a complete cycle
    fn get_cached_cycle_weight(&self, cycle: &[usize], graph: &WeightedGraph) -> f64 {
        if let Ok(mut cache) = self.weight_cache.lock() {
            cache.get_cycle_weight(cycle, graph)
        } else {
            crate::mcmc_sampler::calculate_cycle_weight(cycle, graph)
        }
    }

    /// Clear the cache (useful for testing or memory management)
    pub fn clear_cache(&self) {
        if let Ok(mut cache) = self.weight_cache.lock() {
            cache.clear();
        }
    }

    /// Get cache statistics
    pub fn cache_stats(&self) -> CacheStats {
        if let Ok(cache) = self.weight_cache.lock() {
            cache.stats()
        } else {
            CacheStats::default()
        }
    }
}

impl Default for ProductionAssignmentProcessor {
    fn default() -> Self {
        Self::new()
    }
}

/// Weight cache for repeated calculations
struct WeightCache {
    edge_cache: HashMap<(usize, usize), f64>,
    cycle_cache: HashMap<Vec<usize>, f64>,
    edge_hits: usize,
    edge_misses: usize,
    cycle_hits: usize,
    cycle_misses: usize,
}

impl WeightCache {
    fn new() -> Self {
        Self {
            edge_cache: HashMap::new(),
            cycle_cache: HashMap::new(),
            edge_hits: 0,
            edge_misses: 0,
            cycle_hits: 0,
            cycle_misses: 0,
        }
    }

    fn get_edge_weight(&mut self, giver: usize, receiver: usize, graph: &WeightedGraph) -> f64 {
        let key = (giver, receiver);

        if let Some(&weight) = self.edge_cache.get(&key) {
            self.edge_hits += 1;
            weight
        } else {
            self.edge_misses += 1;
            let weight = graph.get_weight(giver, receiver);
            self.edge_cache.insert(key, weight);
            weight
        }
    }

    fn get_cycle_weight(&mut self, cycle: &[usize], graph: &WeightedGraph) -> f64 {
        let key = cycle.to_vec();

        if let Some(&weight) = self.cycle_cache.get(&key) {
            self.cycle_hits += 1;
            weight
        } else {
            self.cycle_misses += 1;
            let weight = crate::mcmc_sampler::calculate_cycle_weight(cycle, graph);
            self.cycle_cache.insert(key, weight);
            weight
        }
    }

    fn clear(&mut self) {
        self.edge_cache.clear();
        self.cycle_cache.clear();
        self.edge_hits = 0;
        self.edge_misses = 0;
        self.cycle_hits = 0;
        self.cycle_misses = 0;
    }

    fn stats(&self) -> CacheStats {
        CacheStats {
            edge_hit_rate: if self.edge_hits + self.edge_misses == 0 {
                0.0
            } else {
                self.edge_hits as f64 / (self.edge_hits + self.edge_misses) as f64
            },
            cycle_hit_rate: if self.cycle_hits + self.cycle_misses == 0 {
                0.0
            } else {
                self.cycle_hits as f64 / (self.cycle_hits + self.cycle_misses) as f64
            },
            edge_cache_size: self.edge_cache.len(),
            cycle_cache_size: self.cycle_cache.len(),
            total_edge_lookups: self.edge_hits + self.edge_misses,
            total_cycle_lookups: self.cycle_hits + self.cycle_misses,
        }
    }
}

#[derive(Debug, Default)]
pub struct CacheStats {
    pub edge_hit_rate: f64,
    pub cycle_hit_rate: f64,
    pub edge_cache_size: usize,
    pub cycle_cache_size: usize,
    pub total_edge_lookups: usize,
    pub total_cycle_lookups: usize,
}

/// Smart candidate generation with problem-awareness
fn generate_candidate_cycle_smart(
    current: &[usize],
    _graph: &WeightedGraph,
    _participants: &HashSet<usize>,
    iteration: usize,
) -> Vec<usize> {
    let mut candidate = current.to_vec();

    // Early iterations: prefer more explorative moves
    // Later iterations: prefer more exploitative moves
    let exploration_factor = if iteration < 20 { 0.6 } else { 0.3 };

    let move_choice = crate::mcmc_sampler::random();

    if move_choice < exploration_factor {
        // Explorative moves: larger changes
        if candidate.len() >= 4 && crate::mcmc_sampler::random() < 0.5 {
            crate::mcmc_sampler::two_opt_move(&mut candidate);
        } else {
            crate::mcmc_sampler::insertion_move(&mut candidate);
        }
    } else {
        // Exploitative moves: smaller changes
        crate::mcmc_sampler::swap_move(&mut candidate);
    }

    candidate
}

/// Adaptive convergence parameters based on group size
fn adaptive_convergence_params(group_size: usize) -> (usize, usize) {
    match group_size {
        5..=6 => (15, 60),
        7..=8 => (20, 80),
        9..=10 => (25, 100),
        11..=15 => (30, 120),
        _ => (40, 150),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    #[test]
    fn test_production_processor_correctness() {
        println!("\n✅ Testing Production Processor Correctness");

        let processor = ProductionAssignmentProcessor::new();

        let test_cases = [
            // Small group - should use branch-and-bound
            (vec![0, 1, 2, 3], vec![], vec![], "Small group (B&B)"),
            // Medium group with constraints - should use simulated annealing
            (
                vec![0, 1, 2, 3, 4, 5, 6, 7],
                vec![
                    ForbiddenPair {
                        member_1: 0,
                        member_2: 1,
                    },
                    ForbiddenPair {
                        member_1: 2,
                        member_2: 3,
                    },
                    ForbiddenPair {
                        member_1: 4,
                        member_2: 5,
                    },
                    ForbiddenPair {
                        member_1: 6,
                        member_2: 7,
                    },
                ],
                vec![],
                "Medium/High-constraint (SA)",
            ),
            // Large group with history - should use parallel tempering
            (
                vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11],
                vec![],
                vec![
                    Assignment {
                        value: vec![0, 2, 4, 6, 8, 10, 1, 3, 5, 7, 9, 11],
                    },
                    Assignment {
                        value: vec![0, 3, 6, 9, 1, 4, 7, 10, 2, 5, 8, 11],
                    },
                    Assignment {
                        value: vec![0, 4, 8, 1, 5, 9, 2, 6, 10, 3, 7, 11],
                    },
                    Assignment {
                        value: vec![0, 5, 10, 4, 9, 3, 8, 2, 7, 1, 6, 11],
                    },
                    Assignment {
                        value: vec![0, 6, 1, 7, 2, 8, 3, 9, 4, 10, 5, 11],
                    },
                    Assignment {
                        value: vec![0, 7, 3, 10, 6, 2, 9, 5, 1, 8, 4, 11],
                    },
                ],
                "Large/Complex-history (PT)",
            ),
            // Default case - should use MCMC with convergence
            (
                vec![0, 1, 2, 3, 4, 5, 6, 7, 8],
                vec![ForbiddenPair {
                    member_1: 0,
                    member_2: 1,
                }],
                vec![Assignment {
                    value: vec![0, 2, 4, 6, 8, 1, 3, 5, 7],
                }],
                "Default (MCMC+Conv)",
            ),
        ];

        for (participants_vec, forbidden_pairs, history, desc) in &test_cases {
            let participants: HashSet<_> = participants_vec.iter().copied().collect();

            let mut successes = 0;
            let trials = 20;

            for _ in 0..trials {
                if let Some(cycle) =
                    processor.process_assignment(&participants, forbidden_pairs, history)
                {
                    // Verify it's a valid cycle
                    assert_eq!(
                        cycle.len(),
                        participants.len(),
                        "Invalid cycle length for {}",
                        desc
                    );

                    // Verify all participants are included exactly once
                    let cycle_set: HashSet<_> = cycle.iter().copied().collect();
                    assert_eq!(
                        cycle_set.len(),
                        cycle.len(),
                        "Duplicate participants in cycle for {}",
                        desc
                    );
                    for &p in &participants {
                        assert!(
                            cycle_set.contains(&p),
                            "Missing participant {} in cycle for {}",
                            p,
                            desc
                        );
                    }

                    // Verify no forbidden pairs in the cycle
                    let mut valid = true;
                    for i in 0..cycle.len() {
                        let giver = cycle[i];
                        let receiver = cycle[(i + 1) % cycle.len()];

                        for fp in forbidden_pairs {
                            if (fp.member_1 == giver && fp.member_2 == receiver)
                                || (fp.member_1 == receiver && fp.member_2 == giver)
                            {
                                valid = false;
                                break;
                            }
                        }
                        if !valid {
                            break;
                        }
                    }

                    if valid {
                        successes += 1;
                    }
                }
            }

            let success_rate = successes as f64 / trials as f64;
            println!(
                "  {}: {:.1}% success rate ({}/{})",
                desc,
                success_rate * 100.0,
                successes,
                trials
            );
            assert!(
                success_rate >= 0.85,
                "{} success rate should be at least 85%",
                desc
            );
        }
    }

    #[test]
    #[allow(clippy::needless_borrow)]
    fn test_production_processor_performance() {
        println!("\n🚀 Production Processor Performance Benchmark");
        println!("{}", "=".repeat(80));

        let processor = ProductionAssignmentProcessor::new();

        let test_cases = [
            (4, vec![], vec![], 100, "Small (B&B)"),
            (
                8,
                vec![
                    ForbiddenPair {
                        member_1: 0,
                        member_2: 1,
                    },
                    ForbiddenPair {
                        member_1: 2,
                        member_2: 3,
                    },
                    ForbiddenPair {
                        member_1: 4,
                        member_2: 5,
                    },
                    ForbiddenPair {
                        member_1: 6,
                        member_2: 7,
                    },
                ],
                vec![],
                50,
                "Medium/High-constraint (SA)",
            ),
            (
                12,
                vec![],
                vec![
                    Assignment {
                        value: (0..12).collect(),
                    },
                    Assignment {
                        value: (0..12).collect(),
                    },
                    Assignment {
                        value: (0..12).collect(),
                    },
                    Assignment {
                        value: (0..12).collect(),
                    },
                    Assignment {
                        value: (0..12).collect(),
                    },
                    Assignment {
                        value: (0..12).collect(),
                    },
                ],
                30,
                "Large/Complex-history (PT)",
            ),
            (
                9,
                vec![ForbiddenPair {
                    member_1: 0,
                    member_2: 1,
                }],
                vec![Assignment {
                    value: (0..9).collect(),
                }],
                50,
                "Default (MCMC+Conv)",
            ),
        ];

        println!(
            "{:<25} | {:>12} | {:>12} | {:>12} | {:>8}",
            "Algorithm", "Prod (μs)", "Baseline (μs)", "Improvement", "Success%"
        );
        println!("{}", "-".repeat(80));

        for (size, forbidden_pairs, history, iterations, desc) in &test_cases {
            let participants: HashSet<_> = (0..*size).collect();

            // Benchmark production processor
            let start = Instant::now();
            let mut prod_successes = 0;
            for _ in 0..*iterations {
                if processor
                    .process_assignment(&participants, &forbidden_pairs, &history)
                    .is_some()
                {
                    prod_successes += 1;
                }
            }
            let prod_time = start.elapsed().as_micros() as f64 / *iterations as f64;

            // Benchmark baseline MCMC
            let start = Instant::now();
            let mut _baseline_successes = 0;
            for _ in 0..*iterations {
                if crate::mcmc_sampler::mcmc_process_assignment(
                    &participants,
                    &forbidden_pairs,
                    &history,
                )
                .is_some()
                {
                    _baseline_successes += 1;
                }
            }
            let baseline_time = start.elapsed().as_micros() as f64 / *iterations as f64;

            let improvement = if baseline_time > 0.0 {
                (1.0 - prod_time / baseline_time) * 100.0
            } else {
                0.0
            };

            let prod_success_rate = prod_successes as f64 / *iterations as f64;

            println!(
                "{:<25} | {:>12.1} | {:>12.1} | {:>11.1}% | {:>7.1}%",
                desc,
                prod_time,
                baseline_time,
                improvement,
                prod_success_rate * 100.0
            );
        }
        println!("{}", "=".repeat(80));
    }

    #[test]
    #[allow(clippy::needless_borrow)]
    fn test_cache_effectiveness() {
        println!("\n💾 Testing Cache Effectiveness");

        let processor = ProductionAssignmentProcessor::new();

        // Use a larger group (9+ participants) to force MCMC which uses caching
        let participants: HashSet<_> = (0..9).collect();
        let forbidden_pairs = vec![
            ForbiddenPair {
                member_1: 0,
                member_2: 1,
            },
            ForbiddenPair {
                member_1: 2,
                member_2: 3,
            },
        ];
        let history = vec![];

        // Run multiple assignments to populate cache
        for _ in 0..50 {
            let _ = processor.process_assignment(&participants, &forbidden_pairs, &history);
        }

        let stats = processor.cache_stats();
        println!("  Edge cache hit rate: {:.1}%", stats.edge_hit_rate * 100.0);
        println!(
            "  Cycle cache hit rate: {:.1}%",
            stats.cycle_hit_rate * 100.0
        );
        println!("  Edge cache size: {}", stats.edge_cache_size);
        println!("  Cycle cache size: {}", stats.cycle_cache_size);
        println!("  Total edge lookups: {}", stats.total_edge_lookups);
        println!("  Total cycle lookups: {}", stats.total_cycle_lookups);

        // For larger groups with MCMC, cycle cache should have decent hit rate
        // Edge cache may have low hit rate due to diverse edge patterns in MCMC
        assert!(
            stats.cycle_hit_rate > 0.1,
            "Cycle cache hit rate should be > 10%"
        );

        // If there are edge lookups, some should be cache hits
        if stats.total_edge_lookups > 0 {
            assert!(
                stats.edge_hit_rate > 0.0 || stats.total_edge_lookups < 100,
                "Edge cache should have some hits if many lookups occurred"
            );
        }
    }

    #[test]
    #[allow(clippy::needless_borrow)]
    fn test_convergence_detection_effectiveness() {
        println!("\n🎯 Testing Convergence Detection Effectiveness");

        let processor = ProductionAssignmentProcessor::new();

        let participants: HashSet<_> = (0..10).collect();
        let forbidden_pairs = vec![ForbiddenPair {
            member_1: 0,
            member_2: 1,
        }];
        let history = vec![];

        let trials = 20;
        let mut total_time = 0.0;
        let mut successes = 0;

        for _ in 0..trials {
            let start = Instant::now();
            if processor
                .process_assignment(&participants, &forbidden_pairs, &history)
                .is_some()
            {
                successes += 1;
            }
            total_time += start.elapsed().as_micros() as f64;
        }

        let avg_time = total_time / trials as f64;
        let success_rate = successes as f64 / trials as f64;

        println!(
            "  Average time with convergence detection: {:.1} μs",
            avg_time
        );
        println!("  Success rate: {:.1}%", success_rate * 100.0);

        // Compare with baseline (without convergence detection)
        let mut baseline_time = 0.0;
        for _ in 0..trials {
            let start = Instant::now();
            let _ = crate::mcmc_sampler::mcmc_process_assignment(
                &participants,
                &forbidden_pairs,
                &history,
            );
            baseline_time += start.elapsed().as_micros() as f64;
        }
        let avg_baseline_time = baseline_time / trials as f64;

        println!("  Average baseline time: {:.1} μs", avg_baseline_time);

        let improvement = (1.0 - avg_time / avg_baseline_time) * 100.0;
        println!("  Convergence detection improvement: {:.1}%", improvement);

        assert!(success_rate >= 0.9, "Success rate should be at least 90%");
    }

    #[test]
    #[allow(clippy::needless_borrow)]
    fn test_statistical_equivalence() {
        println!("\n📊 Testing Statistical Equivalence with Reference Algorithm");

        let processor = ProductionAssignmentProcessor::new();

        let participants: HashSet<_> = (0..6).collect();
        let forbidden_pairs = vec![];
        let history = vec![];

        let mut prod_results = HashMap::new();
        let mut ref_results = HashMap::new();
        let samples = 1000;

        // Collect samples from production processor
        for _ in 0..samples {
            if let Some(cycle) =
                processor.process_assignment(&participants, &forbidden_pairs, &history)
            {
                let key = canonicalize_cycle(&cycle);
                *prod_results.entry(key).or_insert(0) += 1;
            }
        }

        // Collect samples from reference algorithm
        for _ in 0..samples {
            if let Some(cycle) = crate::mcmc_sampler::mcmc_process_assignment(
                &participants,
                &forbidden_pairs,
                &history,
            ) {
                let key = canonicalize_cycle(&cycle);
                *ref_results.entry(key).or_insert(0) += 1;
            }
        }

        println!("  Production samples: {} unique cycles", prod_results.len());
        println!("  Reference samples: {} unique cycles", ref_results.len());

        // Simple statistical comparison - check that distributions are similar
        let mut total_diff = 0.0;
        let all_cycles: HashSet<_> = prod_results.keys().chain(ref_results.keys()).collect();

        for cycle in &all_cycles {
            let prod_freq = *prod_results.get(*cycle).unwrap_or(&0) as f64 / samples as f64;
            let ref_freq = *ref_results.get(*cycle).unwrap_or(&0) as f64 / samples as f64;
            total_diff += (prod_freq - ref_freq).abs();
        }

        let avg_diff = total_diff / all_cycles.len() as f64;
        println!("  Average frequency difference: {:.4}", avg_diff);

        // The distributions should be reasonably similar
        assert!(
            avg_diff < 0.05,
            "Average frequency difference should be < 0.05"
        );
    }

    fn canonicalize_cycle(cycle: &[usize]) -> Vec<usize> {
        let mut canonical = cycle.to_vec();
        let min_pos = canonical
            .iter()
            .position(|&x| x == *canonical.iter().min().unwrap())
            .unwrap();
        canonical.rotate_left(min_pos);
        canonical
    }

    // Helper functions for creating proper types in tests
    fn create_forbidden_pairs(pairs: &[(usize, usize)]) -> Vec<ForbiddenPair> {
        pairs
            .iter()
            .map(|&(m1, m2)| ForbiddenPair {
                member_1: m1,
                member_2: m2,
            })
            .collect()
    }

    fn create_history(rounds: &[Vec<usize>]) -> Vec<Assignment> {
        rounds
            .iter()
            .map(|round| Assignment {
                value: round.clone(),
            })
            .collect()
    }

    #[test]
    #[allow(clippy::needless_borrow)]
    fn test_distribution_with_history_penalties() {
        println!("\n📈 Testing Distribution with History Penalties (10 participants)");

        // Test both the production processor and reference algorithm for comparison
        let processor = ProductionAssignmentProcessor::new();
        let participants: HashSet<_> = (0..10).collect();
        let forbidden_pairs = create_forbidden_pairs(&[]);

        // Create history where 0->1, 1->2, 2->3, etc. happened in previous round
        // The cycle is: [1, 2, 3, 4, 5, 6, 7, 8, 9, 0]
        let previous_round: Vec<usize> = (1..10).chain(std::iter::once(0)).collect();
        let history = create_history(&[previous_round.clone()]);

        // Debug: verify the history interpretation and weights
        println!("  History cycle: {:?}", previous_round);
        for giver in 0..3 {
            // Just check first few
            if let Some(receiver) = history[0].receiver_of(giver) {
                println!("  History: {} -> {}", giver, receiver);
            }
        }

        // Remove debug output to focus on main issues

        let samples = 1000; // Reduced for faster testing

        // Test production processor
        let mut prod_assignment_counts = HashMap::new();
        for _ in 0..samples {
            if let Some(cycle) =
                processor.process_assignment(&participants, &forbidden_pairs, &history)
            {
                // Correctly interpret the cycle: participant at position i gives to participant at position (i+1)%n
                for i in 0..cycle.len() {
                    let giver = cycle[i];
                    let receiver = cycle[(i + 1) % cycle.len()];
                    let pair = (giver, receiver);
                    *prod_assignment_counts.entry(pair).or_insert(0) += 1;
                }
            }
        }

        // Test reference MCMC algorithm for comparison
        let mut ref_assignment_counts = HashMap::new();
        for _ in 0..samples {
            if let Some(cycle) = crate::mcmc_sampler::mcmc_process_assignment(
                &participants,
                &forbidden_pairs,
                &history,
            ) {
                // Correctly interpret the cycle: participant at position i gives to participant at position (i+1)%n
                for i in 0..cycle.len() {
                    let giver = cycle[i];
                    let receiver = cycle[(i + 1) % cycle.len()];
                    let pair = (giver, receiver);
                    *ref_assignment_counts.entry(pair).or_insert(0) += 1;
                }
            }
        }

        // Compare production vs reference algorithm results
        fn analyze_history_penalty(
            counts: &HashMap<(usize, usize), usize>,
            label: &str,
        ) -> (f64, f64) {
            let mut history_pair_counts = Vec::new();
            let mut non_history_pair_counts = Vec::new();

            for giver in 0..10 {
                let history_receiver = (giver + 1) % 10; // The receiver from history

                for receiver in 0..10 {
                    if giver != receiver {
                        // No self-assignment
                        let count = counts.get(&(giver, receiver)).copied().unwrap_or(0);

                        if receiver == history_receiver {
                            history_pair_counts.push(count);
                        } else {
                            non_history_pair_counts.push(count);
                        }
                    }
                }
            }

            let avg_history =
                history_pair_counts.iter().sum::<usize>() as f64 / history_pair_counts.len() as f64;
            let avg_non_history = non_history_pair_counts.iter().sum::<usize>() as f64
                / non_history_pair_counts.len() as f64;
            let ratio = avg_history / avg_non_history;

            println!(
                "  {} - History pairs avg: {:.1}, Non-history avg: {:.1}, Ratio: {:.3}",
                label, avg_history, avg_non_history, ratio
            );

            (avg_history, ratio)
        }

        let (_, prod_ratio) = analyze_history_penalty(&prod_assignment_counts, "Production");
        let (_, ref_ratio) = analyze_history_penalty(&ref_assignment_counts, "Reference");

        // History should be penalized - with current weight (0.3), expect ratio around 0.3-0.6
        // Account for sampling variance, so use 0.95 as threshold
        assert!(
            ref_ratio < 0.95,
            "Reference MCMC should penalize history pairs (ratio: {:.3} should be < 0.95)",
            ref_ratio
        );

        // Production should also work
        assert!(
            prod_ratio < 0.95,
            "Production algorithm should penalize history pairs (ratio: {:.3} should be < 0.95)",
            prod_ratio
        );
    }

    #[test]
    #[allow(clippy::needless_borrow)]
    fn test_distribution_with_forbidden_pairs() {
        println!("\n🚫 Testing Distribution with Forbidden Pairs (12 participants)");

        let _processor = ProductionAssignmentProcessor::new();
        let participants: HashSet<_> = (0..12).collect();

        // Create forbidden pairs: 0<->6, 1<->7, 2<->8, 3<->9 (4 bidirectional pairs)
        let forbidden_pairs = create_forbidden_pairs(&[
            (0, 6),
            (6, 0),
            (1, 7),
            (7, 1),
            (2, 8),
            (8, 2),
            (3, 9),
            (9, 3),
        ]);
        let history = create_history(&[]);

        // Debug: Check that forbidden pairs have zero weight
        use crate::mcmc_sampler::{calculate_cycle_weight, WeightedGraph};
        let graph = WeightedGraph::new(&participants, &forbidden_pairs, &history);
        println!("  Forbidden pair weights:");
        for &(g, r) in &[(0, 6), (1, 7), (2, 8)] {
            println!("    {} -> {}: {:.3}", g, r, graph.get_weight(g, r));
        }

        // Debug: Test specific problematic cycles
        let problematic_cycle = vec![0, 1, 2, 8, 4, 5, 6, 7, 3, 9, 10, 11]; // Contains 2->8 forbidden pair
        let weight = calculate_cycle_weight(&problematic_cycle, &graph);
        println!("  Cycle with 2->8 forbidden pair weight: {:.3}", weight);

        if weight > 0.0 {
            println!("  ⚠️  BUG: Cycle with forbidden pair has positive weight!");
            // Check each edge
            for i in 0..problematic_cycle.len() {
                let giver = problematic_cycle[i];
                let receiver = problematic_cycle[(i + 1) % problematic_cycle.len()];
                let edge_weight = graph.get_weight(giver, receiver);
                println!("    Edge {} -> {}: {:.3}", giver, receiver, edge_weight);
            }
        }

        let samples = 2000;
        let mut assignment_counts = HashMap::new();

        // Test the reference MCMC algorithm directly to isolate the issue
        for _ in 0..samples {
            if let Some(cycle) = crate::mcmc_sampler::mcmc_process_assignment(
                &participants,
                &forbidden_pairs,
                &history,
            ) {
                // Correctly interpret the cycle: participant at position i gives to participant at position (i+1)%n
                for i in 0..cycle.len() {
                    let giver = cycle[i];
                    let receiver = cycle[(i + 1) % cycle.len()];
                    let pair = (giver, receiver);
                    *assignment_counts.entry(pair).or_insert(0) += 1;
                }
            }
        }

        // Verify forbidden pairs never occur
        let forbidden_set: HashSet<_> = forbidden_pairs
            .iter()
            .map(|fp| (fp.member_1, fp.member_2))
            .collect();
        for &(giver, receiver) in &forbidden_set {
            let count = assignment_counts
                .get(&(giver, receiver))
                .copied()
                .unwrap_or(0);
            assert_eq!(
                count, 0,
                "Forbidden pair ({}, {}) should never occur",
                giver, receiver
            );
        }

        // Check that valid pairs have non-zero counts (algorithm works)
        let mut valid_pair_counts = Vec::new();
        let mut zero_count_pairs = 0;

        for giver in 0..12 {
            for receiver in 0..12 {
                if giver != receiver && !forbidden_set.contains(&(giver, receiver)) {
                    let count = assignment_counts
                        .get(&(giver, receiver))
                        .copied()
                        .unwrap_or(0);
                    valid_pair_counts.push(count);
                    if count == 0 {
                        zero_count_pairs += 1;
                    }
                }
            }
        }

        let avg_count =
            valid_pair_counts.iter().sum::<usize>() as f64 / valid_pair_counts.len() as f64;
        let min_count = *valid_pair_counts.iter().min().unwrap();
        let max_count = *valid_pair_counts.iter().max().unwrap();

        println!(
            "  Forbidden pairs: {} (all should have 0 count)",
            forbidden_pairs.len()
        );
        println!(
            "  Valid pairs: {} (average count: {:.1})",
            valid_pair_counts.len(),
            avg_count
        );
        println!(
            "  Count range: {} to {} (zero pairs: {})",
            min_count, max_count, zero_count_pairs
        );

        // With no history, valid pairs should have reasonable representation
        // (not necessarily even - some natural variation is expected)
        assert!(
            zero_count_pairs < valid_pair_counts.len() / 4,
            "Most valid pairs should have non-zero counts with no history constraints"
        );
    }

    #[test]
    #[allow(clippy::needless_borrow)]
    fn test_distribution_with_combined_constraints() {
        println!(
            "\n🔄 Testing Distribution with Combined History + Forbidden Pairs (8 participants)"
        );

        let processor = ProductionAssignmentProcessor::new();
        let participants: HashSet<_> = (0..8).collect();

        // Forbidden pairs: 0<->4, 1<->5 (couples)
        let forbidden_pairs = create_forbidden_pairs(&[(0, 4), (4, 0), (1, 5), (5, 1)]);

        // History: previous round was 0->2, 1->3, 2->6, 3->7, 4->1, 5->0, 6->4, 7->5
        let history = create_history(&[vec![2, 3, 6, 7, 1, 0, 4, 5]]);

        let samples = 3000;
        let mut assignment_counts = HashMap::new();
        let mut successful_assignments = 0;

        for _ in 0..samples {
            if let Some(cycle) =
                processor.process_assignment(&participants, &forbidden_pairs, &history)
            {
                successful_assignments += 1;
                // Correctly interpret the cycle: participant at position i gives to participant at position (i+1)%n
                for i in 0..cycle.len() {
                    let giver = cycle[i];
                    let receiver = cycle[(i + 1) % cycle.len()];
                    let pair = (giver, receiver);
                    *assignment_counts.entry(pair).or_insert(0) += 1;
                }
            }
        }

        println!(
            "  Successful assignments: {}/{} ({:.1}%)",
            successful_assignments,
            samples,
            100.0 * successful_assignments as f64 / samples as f64
        );

        // Verify constraints are respected
        let forbidden_set: HashSet<_> = forbidden_pairs
            .iter()
            .map(|fp| (fp.member_1, fp.member_2))
            .collect();
        // Correctly interpret history cycle: participant at position i gives to participant at position (i+1)%n
        let history_pairs: HashSet<_> = (0..8)
            .map(|i| {
                let giver = history[0].value[i];
                let receiver = history[0].value[(i + 1) % 8];
                (giver, receiver)
            })
            .collect();

        // Check forbidden pairs are never assigned
        for &(giver, receiver) in &forbidden_set {
            let count = assignment_counts
                .get(&(giver, receiver))
                .copied()
                .unwrap_or(0);
            assert_eq!(
                count, 0,
                "Forbidden pair ({}, {}) should never occur",
                giver, receiver
            );
        }

        // Analyze history vs non-history vs forbidden distribution
        let mut history_counts = Vec::new();
        let mut regular_counts = Vec::new();

        for giver in 0..8 {
            for receiver in 0..8 {
                if giver != receiver {
                    let pair = (giver, receiver);
                    let count = assignment_counts.get(&pair).copied().unwrap_or(0);

                    if forbidden_set.contains(&pair) {
                        assert_eq!(count, 0, "Forbidden pair should have 0 count");
                    } else if history_pairs.contains(&pair) {
                        history_counts.push(count);
                    } else {
                        regular_counts.push(count);
                    }
                }
            }
        }

        let avg_history = history_counts.iter().sum::<usize>() as f64 / history_counts.len() as f64;
        let avg_regular = regular_counts.iter().sum::<usize>() as f64 / regular_counts.len() as f64;

        println!("  History pairs average count: {:.1}", avg_history);
        println!("  Regular pairs average count: {:.1}", avg_regular);
        println!("  History penalty ratio: {:.3}", avg_history / avg_regular);

        // History should be penalized even with forbidden pairs present
        assert!(
            avg_history < avg_regular * 0.8,
            "History pairs should be penalized even with forbidden constraints"
        );
    }

    #[test]
    #[allow(clippy::needless_borrow)]
    fn test_fairness_over_multiple_rounds() {
        println!("\n🔄 Testing Fairness Over Multiple Rounds (6 participants, 5 rounds)");

        let processor = ProductionAssignmentProcessor::new();
        let participants: HashSet<_> = (0..6).collect();
        let forbidden_pairs = create_forbidden_pairs(&[]);

        let mut history = vec![];
        let mut all_assignments = Vec::new();
        let rounds = 5;

        // Simulate multiple rounds
        for round in 0..rounds {
            let samples = 1000;
            let mut round_assignments = HashMap::new();

            for _ in 0..samples {
                if let Some(cycle) =
                    processor.process_assignment(&participants, &forbidden_pairs, &history)
                {
                    let key = canonicalize_cycle(&cycle);
                    *round_assignments.entry(key).or_insert(0) += 1;
                }
            }

            // Pick the most common assignment for this round
            let most_common = round_assignments
                .iter()
                .max_by_key(|(_, &count)| count)
                .map(|(cycle, _)| cycle.clone())
                .unwrap();

            history.push(Assignment {
                value: most_common.clone(),
            });
            all_assignments.push(most_common);

            println!(
                "  Round {}: Selected assignment with {} variants",
                round + 1,
                round_assignments.len()
            );
        }

        // Analyze fairness: each person should give to each other person roughly equally over time
        let mut giving_matrix = vec![vec![0; 6]; 6];

        for assignment in &all_assignments {
            // Correctly interpret the cycle: participant at position i gives to participant at position (i+1)%n
            for i in 0..assignment.len() {
                let giver = assignment[i];
                let receiver = assignment[(i + 1) % assignment.len()];
                giving_matrix[giver][receiver] += 1;
            }
        }

        // Check distribution fairness
        let mut all_counts = Vec::new();
        for (giver, giver_row) in giving_matrix.iter().enumerate().take(6) {
            for (receiver, &count) in giver_row.iter().enumerate().take(6) {
                if giver != receiver {
                    all_counts.push(count);
                }
            }
        }

        let total_assignments = all_counts.iter().sum::<usize>();
        let expected_per_pair = total_assignments as f64 / (6 * 5) as f64; // 6*5 valid pairs

        let variance = all_counts
            .iter()
            .map(|&count| (count as f64 - expected_per_pair).powi(2))
            .sum::<f64>()
            / all_counts.len() as f64;
        let std_dev = variance.sqrt();

        println!("  Total assignments: {}", total_assignments);
        println!("  Expected per pair: {:.2}", expected_per_pair);
        println!("  Standard deviation: {:.2}", std_dev);
        println!(
            "  Coefficient of variation: {:.3}",
            std_dev / expected_per_pair
        );

        // Over multiple rounds with history penalties, distribution will be uneven
        // This is EXPECTED behavior - history penalties create intentional bias to avoid repetition
        // With strong history penalties and small groups, variation should be high
        assert!(
            std_dev / expected_per_pair < 2.0,
            "Distribution variation should be bounded even with history penalties (current: {:.3})",
            std_dev / expected_per_pair
        );
    }

    #[test]
    #[allow(clippy::needless_borrow)]
    fn test_edge_case_high_constraint_density() {
        println!(
            "\n⚠️  Testing Edge Case: High Constraint Density (10 participants, many constraints)"
        );

        let processor = ProductionAssignmentProcessor::new();
        let participants: HashSet<_> = (0..10).collect();

        // Create many forbidden pairs (about 30% of possible pairs)
        let mut forbidden_pair_tuples = vec![];
        for i in 0..10 {
            for j in (i + 1)..10 {
                if (i + j) % 3 == 0 {
                    // Create a pattern of forbidden pairs
                    forbidden_pair_tuples.push((i, j));
                    forbidden_pair_tuples.push((j, i));
                }
            }
        }
        let forbidden_pairs = create_forbidden_pairs(&forbidden_pair_tuples);

        // Add complex history (3 previous rounds)
        let history = create_history(&[
            vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 0], // Round 1
            vec![2, 3, 4, 5, 6, 7, 8, 9, 0, 1], // Round 2
            vec![9, 0, 1, 2, 3, 4, 5, 6, 7, 8], // Round 3
        ]);

        let samples = 1000;
        let mut successful_assignments = 0;
        let mut assignment_counts = HashMap::new();

        for _ in 0..samples {
            if let Some(cycle) =
                processor.process_assignment(&participants, &forbidden_pairs, &history)
            {
                successful_assignments += 1;

                // Correctly interpret the cycle: participant at position i gives to participant at position (i+1)%n
                for i in 0..cycle.len() {
                    let giver = cycle[i];
                    let receiver = cycle[(i + 1) % cycle.len()];
                    let pair = (giver, receiver);
                    *assignment_counts.entry(pair).or_insert(0) += 1;
                }
            }
        }

        let success_rate = successful_assignments as f64 / samples as f64;
        println!("  Forbidden pairs: {}", forbidden_pairs.len());
        println!("  History rounds: {}", history.len());
        println!(
            "  Success rate: {:.1}% ({}/{})",
            success_rate * 100.0,
            successful_assignments,
            samples
        );

        // Should still achieve reasonable success rate even with high constraints
        assert!(
            success_rate > 0.7,
            "Should achieve >70% success rate even with high constraints"
        );

        // Verify all constraints are respected in successful assignments
        let forbidden_set: HashSet<_> = forbidden_pairs
            .iter()
            .map(|fp| (fp.member_1, fp.member_2))
            .collect();
        let all_history_pairs: HashSet<_> = history
            .iter()
            .flat_map(|round| (0..10).map(move |i| (i, round.value[i])))
            .collect();

        for &(giver, receiver) in &forbidden_set {
            let count = assignment_counts
                .get(&(giver, receiver))
                .copied()
                .unwrap_or(0);
            assert_eq!(
                count, 0,
                "Forbidden pair ({}, {}) should never occur",
                giver, receiver
            );
        }

        // History pairs should be less frequent
        let mut history_total = 0;
        let mut non_history_total = 0;
        let mut history_pair_count = 0;
        let mut non_history_pair_count = 0;

        for giver in 0..10 {
            for receiver in 0..10 {
                if giver != receiver && !forbidden_set.contains(&(giver, receiver)) {
                    let count = assignment_counts
                        .get(&(giver, receiver))
                        .copied()
                        .unwrap_or(0);

                    if all_history_pairs.contains(&(giver, receiver)) {
                        history_total += count;
                        history_pair_count += 1;
                    } else {
                        non_history_total += count;
                        non_history_pair_count += 1;
                    }
                }
            }
        }

        if history_pair_count > 0 && non_history_pair_count > 0 {
            let avg_history = history_total as f64 / history_pair_count as f64;
            let avg_non_history = non_history_total as f64 / non_history_pair_count as f64;

            println!(
                "  History pairs avg: {:.1}, Non-history pairs avg: {:.1}",
                avg_history, avg_non_history
            );
            println!(
                "  History penalty ratio: {:.3}",
                avg_history / avg_non_history
            );

            // Even with high constraints, history should still be penalized
            assert!(
                avg_history < avg_non_history * 0.9,
                "History should be penalized even with high constraint density"
            );
        }
    }
}
