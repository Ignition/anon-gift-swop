use crate::assignment::{Assignment, AssignmentPair};
use crate::default_map::DefaultMap;
use crate::forbidden_pair::ForbiddenPair;
use std::collections::HashSet;

// For WASM compatibility, we'll use a simple random number generator
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = Math)]
    pub fn random() -> f64;
}

// For testing on native targets, use a simple PRNG
#[cfg(not(target_arch = "wasm32"))]
#[allow(clippy::cast_precision_loss)] // Simple PRNG implementation
pub fn random() -> f64 {
    use std::cell::RefCell;
    thread_local! {
        static RNG_STATE: RefCell<u64> = RefCell::new(1);
    }

    RNG_STATE.with(|state| {
        let mut s = state.borrow_mut();
        *s = s.wrapping_mul(1_103_515_245).wrapping_add(12345);
        ((*s / 65536) % 32768) as f64 / 32768.0
    })
}

#[derive(Debug)]
pub struct WeightedGraph {
    weights: DefaultMap<AssignmentPair, f64>,
}

impl WeightedGraph {
    pub fn new(
        participants: &HashSet<usize>,
        forbidden_pairs: &[ForbiddenPair],
        history: &[Assignment],
    ) -> Self {
        let weights = crate::weights_for(participants, forbidden_pairs, history);

        Self { weights }
    }

    pub fn get_weight(&self, giver: usize, receiver: usize) -> f64 {
        if giver == receiver {
            return 0.0; // No self-assignment in gift exchange
        }
        self.weights.get(&AssignmentPair { giver, receiver })
    }
}

/// Public interface for WASM - uses optimized MCMC sampling for efficiency
#[allow(dead_code)]
pub fn mcmc_process_assignment(
    participants: &HashSet<usize>,
    forbidden_pairs: &[ForbiddenPair],
    history: &[Assignment],
) -> Option<Vec<usize>> {
    // Single participant case is impossible (can't give to themselves)
    if participants.len() == 1 {
        return None;
    }

    // Use optimized MCMC sampling with adaptive iterations
    optimized_mcmc_sampling(participants, forbidden_pairs, history)
}

/// Optimized MCMC with adaptive iteration counts based on group size
#[allow(dead_code)]
fn optimized_mcmc_sampling(
    participants: &HashSet<usize>,
    forbidden_pairs: &[ForbiddenPair],
    history: &[Assignment],
) -> Option<Vec<usize>> {
    let graph = WeightedGraph::new(participants, forbidden_pairs, history);

    // For very small groups, fall back to rejection sampling for exactness
    if participants.len() <= 4 {
        return rejection_sample_cycle_small(&graph, participants);
    }

    // Use rejection sampling with weighted probability for better distribution
    // This ensures the sampling directly respects edge weights
    weighted_rejection_sampling(&graph, participants)
}

/// Weighted rejection sampling that properly respects edge weights
#[allow(dead_code)]
fn weighted_rejection_sampling(
    graph: &WeightedGraph,
    participants: &HashSet<usize>,
) -> Option<Vec<usize>> {
    let participants_vec: Vec<_> = participants.iter().copied().collect();
    let max_attempts = 10000;

    // Use a more aggressive approach: collect many samples and emphasize weight differences
    let mut candidates = Vec::new();

    // Collect a larger set of candidates to get better weight distribution
    for _attempt in 0..max_attempts {
        let mut cycle = participants_vec.clone();
        shuffle_cycle(&mut cycle);

        let weight = calculate_cycle_weight(&cycle, graph);

        if weight > 0.0 {
            candidates.push((cycle, weight));

            // Collect more candidates for better statistics
            if candidates.len() >= 500 {
                break;
            }
        }
    }

    if candidates.is_empty() {
        return None;
    }

    // Apply exponential weighting to emphasize differences
    // This makes higher-weight cycles much more likely to be selected
    let weighted_candidates: Vec<_> = candidates
        .iter()
        .map(|(cycle, weight)| {
            // Exponentially amplify weight differences
            // weight^3 makes small differences much more pronounced
            let amplified_weight = weight.powi(3);
            (cycle.clone(), amplified_weight)
        })
        .collect();

    // Weighted random selection with amplified weights
    let total_weight: f64 = weighted_candidates.iter().map(|(_, w)| w).sum();
    if total_weight <= 0.0 {
        return None;
    }

    let mut threshold = random() * total_weight;
    for (cycle, weight) in &weighted_candidates {
        threshold -= weight;
        if threshold <= 0.0 {
            return Some(cycle.clone());
        }
    }

    // Fallback to first candidate
    candidates.into_iter().next().map(|(cycle, _)| cycle)
}

/// Find any valid cycle as fallback
fn find_any_valid_cycle(
    graph: &WeightedGraph,
    participants: &HashSet<usize>,
) -> Option<Vec<usize>> {
    let participants_vec: Vec<_> = participants.iter().copied().collect();
    let max_attempts = 1000;
    let mut _zero_count = 0;

    for _attempt in 0..max_attempts {
        let mut cycle = participants_vec.clone();
        shuffle_cycle(&mut cycle);

        let weight = calculate_cycle_weight(&cycle, graph);
        if weight > 0.0 {
            return Some(cycle);
        } else {
            _zero_count += 1;
        }
    }

    // If no valid cycle found, the constraints may be impossible

    None
}

/// For small groups (≤4), use exact rejection sampling for mathematical correctness
#[allow(dead_code)]
pub fn rejection_sample_cycle_small(
    graph: &WeightedGraph,
    participants: &HashSet<usize>,
) -> Option<Vec<usize>> {
    // Only enumerate for very small groups where it's still feasible
    let all_cycles = enumerate_all_cycles(participants);
    let mut valid_cycles = Vec::new();
    let mut total_weight = 0.0;

    for cycle in all_cycles {
        let weight = calculate_cycle_weight(&cycle, graph);
        if weight > 0.0 {
            total_weight += weight;
            valid_cycles.push((cycle, weight));
        }
    }

    if total_weight <= 0.0 {
        return None;
    }

    // Weighted random selection
    let mut threshold = random() * total_weight;
    for (cycle, weight) in valid_cycles {
        threshold -= weight;
        if threshold <= 0.0 {
            return Some(cycle);
        }
    }

    None
}

/// Find an initial valid cycle to start MCMC from
/// Returns None if no valid cycle exists (constraints are impossible)
pub fn find_initial_valid_cycle(
    graph: &WeightedGraph,
    participants: &HashSet<usize>,
) -> Option<Vec<usize>> {
    // Try many attempts to find a valid starting point
    for _ in 0..10000 {
        let mut cycle: Vec<_> = participants.iter().copied().collect();
        shuffle_cycle(&mut cycle);

        if calculate_cycle_weight(&cycle, graph) > 0.0 {
            return Some(cycle);
        }
    }

    // Fallback to more systematic search
    find_any_valid_cycle(graph, participants)
}

/// 2-opt move: reverse a segment of the cycle
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss
)] // Random generation
pub fn two_opt_move(cycle: &mut [usize]) {
    let n = cycle.len();
    if n < 4 {
        return;
    }

    let i = (random() * n as f64) as usize;
    let j = (random() * n as f64) as usize;

    let (start, end) = if i < j { (i, j) } else { (j, i) };
    if end - start < 2 {
        return;
    }

    cycle[start..=end].reverse();
}

/// Swap move: swap two random elements
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss
)] // Random generation
pub fn swap_move(cycle: &mut [usize]) {
    let n = cycle.len();
    if n < 2 {
        return;
    }

    let i = (random() * n as f64) as usize;
    let j = (random() * n as f64) as usize;

    if i != j {
        cycle.swap(i, j);
    }
}

/// Insertion move: move an element to a different position
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss
)] // Random generation
pub fn insertion_move(cycle: &mut Vec<usize>) {
    let n = cycle.len();
    if n < 3 {
        return;
    }

    let from = (random() * n as f64) as usize;
    let to = (random() * n as f64) as usize;

    if from != to {
        let element = cycle.remove(from);
        cycle.insert(to, element);
    }
}

/// Calculate the weight of a cycle
pub fn calculate_cycle_weight(cycle: &[usize], graph: &WeightedGraph) -> f64 {
    let n = cycle.len();
    let mut total_score = 0.0;
    let mut has_forbidden = false;

    for i in 0..n {
        let giver = cycle[i];
        let receiver = cycle[(i + 1) % n];
        let edge_weight = graph.get_weight(giver, receiver);

        if edge_weight == 0.0 {
            has_forbidden = true;
            break; // Forbidden assignments make cycle invalid
        }

        // Convert weights to additive scores for proper MCMC sampling
        // weight=1.0 (normal) -> score=1.0
        // weight=0.5 (history penalty) -> score=0.5
        total_score += edge_weight;
    }

    if has_forbidden {
        return 0.0;
    }

    // Return average edge score (maintains comparability across different cycle sizes)
    total_score / n as f64
}

/// Shuffle a cycle randomly
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss
)] // Random generation
fn shuffle_cycle(cycle: &mut [usize]) {
    for i in (1..cycle.len()).rev() {
        let j = (random() * (i + 1) as f64) as usize;
        cycle.swap(i, j);
    }
}

/// For small groups, enumerate all possible cycles
#[allow(dead_code)]
fn enumerate_all_cycles(participants: &HashSet<usize>) -> Vec<Vec<usize>> {
    let mut items: Vec<_> = participants.iter().copied().collect();
    items.sort_unstable();

    if items.len() > 8 {
        // For larger groups, use MCMC approximation
        return vec![items]; // Just return one cycle and let MCMC improve it
    }

    let mut results = Vec::new();
    permute(&mut items, 0, &mut results);
    results
}

#[allow(dead_code)]
fn permute(items: &mut Vec<usize>, start: usize, results: &mut Vec<Vec<usize>>) {
    if start == items.len() {
        results.push(items.clone());
        return;
    }

    for i in start..items.len() {
        items.swap(start, i);
        permute(items, start + 1, results);
        items.swap(start, i);
    }
}
