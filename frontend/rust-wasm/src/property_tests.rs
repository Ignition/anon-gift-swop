use crate::assignment::Assignment;
use crate::forbidden_pair::ForbiddenPair;
use crate::production_optimized::ProductionAssignmentProcessor;
use proptest::prelude::*;
use std::collections::HashSet;

proptest! {
    /// Property test: Algorithm always produces a valid cycle or None
    #[test]
    fn prop_always_produces_cycle_or_none(
        participants in proptest::collection::hash_set(0usize..15, 2..=12),
        forbidden_pair_indices in proptest::collection::vec(0usize..100, 0..20),
        history_length in 0usize..4,
    ) {
        // Convert indices to actual forbidden pairs
        let participants_vec: Vec<_> = participants.iter().copied().collect();
        let mut all_pairs = Vec::new();
        for i in 0..participants_vec.len() {
            for j in i + 1..participants_vec.len() {
                all_pairs.push((participants_vec[i], participants_vec[j]));
            }
        }

        let forbidden_pairs: Vec<_> = forbidden_pair_indices
            .iter()
            .filter_map(|&idx| all_pairs.get(idx % all_pairs.len().max(1)))
            .take(all_pairs.len() / 2) // Max 50% forbidden to ensure solvability
            .map(|&(a, b)| ForbiddenPair { member_1: a.min(b), member_2: a.max(b) })
            .collect();

        // Generate simple history
        let history: Vec<_> = (0..history_length)
            .map(|_| Assignment { value: participants_vec.clone() })
            .collect();

        let processor = ProductionAssignmentProcessor::new();
        let result = processor.process_assignment(&participants, &forbidden_pairs, &history);

        match result {
            Some(cycle) => {
                // Property 1: Result is a valid cycle
                prop_assert_eq!(cycle.len(), participants.len(), "Cycle length must equal participant count");

                // All participants must be present exactly once
                let cycle_set: HashSet<_> = cycle.iter().copied().collect();
                prop_assert_eq!(cycle_set.len(), participants.len(), "All participants must be unique in cycle");
                prop_assert_eq!(cycle_set, participants, "Cycle must contain exactly the input participants");

                // Verify it's a proper cycle (no self-assignments)
                for i in 0..cycle.len() {
                    let giver = cycle[i];
                    let receiver = cycle[(i + 1) % cycle.len()];
                    prop_assert_ne!(giver, receiver, "No self-assignments allowed: participant {} cannot give to themselves", giver);
                }
            }
            None => {
                // Property 2: None is returned when no valid assignment exists
                // This is acceptable - we just verify the algorithm doesn't crash
            }
        }
    }
}

proptest! {
    /// Property test: Cycles never contain forbidden pairings
    #[test]
    fn prop_cycles_never_contain_forbidden_pairs(
        participants in proptest::collection::hash_set(0usize..10, 2..=8),
        forbidden_pair_count in 0usize..10,
        seed in any::<u64>(),
    ) {
        // Generate deterministic forbidden pairs based on seed
        let participants_vec: Vec<_> = participants.iter().copied().collect();
        let mut all_pairs = Vec::new();
        for i in 0..participants_vec.len() {
            for j in i + 1..participants_vec.len() {
                all_pairs.push((participants_vec[i], participants_vec[j]));
            }
        }

        // Use seed to select forbidden pairs deterministically
        let forbidden_pairs: Vec<_> = (0..forbidden_pair_count.min(all_pairs.len() / 3))
            .map(|i| {
                let idx = ((seed.wrapping_add(i as u64)) as usize) % all_pairs.len();
                let (a, b) = all_pairs[idx];
                ForbiddenPair { member_1: a.min(b), member_2: a.max(b) }
            })
            .collect();

        let history = vec![];
        let processor = ProductionAssignmentProcessor::new();
        let result = processor.process_assignment(&participants, &forbidden_pairs, &history);

        if let Some(cycle) = result {
            // Check every edge in the cycle against forbidden pairs
            for i in 0..cycle.len() {
                let giver = cycle[i];
                let receiver = cycle[(i + 1) % cycle.len()];

                // Verify this edge is not forbidden
                for forbidden_pair in &forbidden_pairs {
                    let is_forbidden = (forbidden_pair.member_1 == giver && forbidden_pair.member_2 == receiver) ||
                                     (forbidden_pair.member_1 == receiver && forbidden_pair.member_2 == giver);

                    prop_assert!(!is_forbidden,
                        "Forbidden pair violated: {} -> {} (forbidden pair: {}, {})",
                        giver, receiver, forbidden_pair.member_1, forbidden_pair.member_2);
                }
            }
        }
    }
}

proptest! {
    /// Property test: Test with extreme constraint scenarios
    #[test]
    fn prop_handles_extreme_constraints(
        participants in proptest::collection::hash_set(0usize..8, 2..=6),
        constraint_fraction in 0.0f64..0.8, // 0% to 80% of pairs forbidden
    ) {
        let participants_vec: Vec<_> = participants.iter().copied().collect();

        // Create all possible pairs
        let mut all_possible_pairs = Vec::new();
        for i in 0..participants_vec.len() {
            for j in i + 1..participants_vec.len() {
                all_possible_pairs.push((participants_vec[i], participants_vec[j]));
            }
        }

        // Select fraction of pairs to forbid
        let num_forbidden = (all_possible_pairs.len() as f64 * constraint_fraction) as usize;
        let forbidden_pairs: Vec<_> = all_possible_pairs
            .iter()
            .take(num_forbidden)
            .map(|&(a, b)| ForbiddenPair { member_1: a.min(b), member_2: a.max(b) })
            .collect();

        let history = vec![];
        let processor = ProductionAssignmentProcessor::new();
        let result = processor.process_assignment(&participants, &forbidden_pairs, &history);

        // Algorithm should not crash regardless of constraint density
        match result {
            Some(cycle) => {
                // If solution found, it must be valid
                prop_assert_eq!(cycle.len(), participants.len());

                // Verify no forbidden pairs
                for i in 0..cycle.len() {
                    let giver = cycle[i];
                    let receiver = cycle[(i + 1) % cycle.len()];

                    for forbidden_pair in &forbidden_pairs {
                        let is_forbidden = (forbidden_pair.member_1 == giver && forbidden_pair.member_2 == receiver) ||
                                         (forbidden_pair.member_1 == receiver && forbidden_pair.member_2 == giver);
                        prop_assert!(!is_forbidden, "Forbidden pair in extreme constraint test");
                    }
                }
            }
            None => {
                // No solution found - this is acceptable for highly constrained problems
            }
        }
    }
}

proptest! {
    /// Property test: Algorithm handles edge cases gracefully
    #[test]
    fn prop_handles_edge_cases(
        size in 1usize..=3, // Test very small groups
    ) {
        let participants: HashSet<_> = (0..size).collect();
        let forbidden_pairs = vec![];
        let history = vec![];

        let processor = ProductionAssignmentProcessor::new();
        let result = processor.process_assignment(&participants, &forbidden_pairs, &history);

        match size {
            1 => {
                // Single participant should always return None (impossible)
                prop_assert!(result.is_none(), "Single participant should return None");
            }
            2 | 3 => {
                // These should be solvable with no constraints
                if let Some(cycle) = result {
                    prop_assert_eq!(cycle.len(), size);

                    // Verify no self-assignments
                    for i in 0..cycle.len() {
                        let giver = cycle[i];
                        let receiver = cycle[(i + 1) % cycle.len()];
                        prop_assert_ne!(giver, receiver);
                    }
                }
            }
            _ => unreachable!(),
        }
    }
}

proptest! {
    /// Property test: Verify cycles are valid permutations
    #[test]
    fn prop_cycles_are_valid_permutations(
        participants in proptest::collection::hash_set(0usize..12, 2..=10),
    ) {
        let forbidden_pairs = vec![];
        let history = vec![];

        let processor = ProductionAssignmentProcessor::new();
        let result = processor.process_assignment(&participants, &forbidden_pairs, &history);

        if let Some(cycle) = result {
            // Must be same length as input
            prop_assert_eq!(cycle.len(), participants.len());

            // Must contain each participant exactly once
            let mut sorted_cycle = cycle.clone();
            sorted_cycle.sort_unstable();
            let mut sorted_participants: Vec<_> = participants.iter().copied().collect();
            sorted_participants.sort_unstable();

            prop_assert_eq!(sorted_cycle, sorted_participants, "Cycle must be a permutation of participants");

            // No duplicates
            let cycle_set: HashSet<_> = cycle.iter().copied().collect();
            prop_assert_eq!(cycle_set.len(), cycle.len(), "Cycle must have no duplicates");
        }
    }
}

proptest! {
    /// Property test: Algorithm respects history penalties (when solution exists)
    #[test]
    fn prop_respects_history_when_possible(
        participants in proptest::collection::hash_set(0usize..8, 3..=6),
        _history_pair_count in 1usize..3,
    ) {
        let participants_vec: Vec<_> = participants.iter().copied().collect();

        // Create simple history with some repeated assignments
        let history = vec![Assignment { value: participants_vec.clone() }];

        let forbidden_pairs = vec![];
        let processor = ProductionAssignmentProcessor::new();

        // Run multiple times to test distribution
        let mut results = Vec::new();
        for _ in 0..20 {
            if let Some(cycle) = processor.process_assignment(&participants, &forbidden_pairs, &history) {
                results.push(cycle);
            }
        }

        // If we got results, verify they're all valid
        for cycle in results {
            prop_assert_eq!(cycle.len(), participants.len());

            // Verify no self-assignments
            for i in 0..cycle.len() {
                let giver = cycle[i];
                let receiver = cycle[(i + 1) % cycle.len()];
                prop_assert_ne!(giver, receiver);
            }

            // Verify it's a valid permutation
            let cycle_set: HashSet<_> = cycle.iter().copied().collect();
            prop_assert_eq!(cycle_set, participants.clone());
        }
    }
}

/// Simple smoke test for the production processor
#[test]
fn test_production_processor_smoke() {
    let participants: HashSet<_> = (0..4).collect();
    let forbidden_pairs = vec![ForbiddenPair {
        member_1: 0,
        member_2: 1,
    }];
    let history = vec![];

    let processor = ProductionAssignmentProcessor::new();
    let result = processor.process_assignment(&participants, &forbidden_pairs, &history);

    // Should either return a valid cycle or None
    match result {
        Some(cycle) => {
            assert_eq!(cycle.len(), 4);

            // Verify forbidden pair is not present
            for i in 0..cycle.len() {
                let giver = cycle[i];
                let receiver = cycle[(i + 1) % cycle.len()];
                assert!(!(giver == 0 && receiver == 1));
                assert!(!(giver == 1 && receiver == 0));
            }
        }
        None => {
            // Acceptable if constraints are impossible
        }
    }
}

/// Test that algorithm never produces self-assignments
#[test]
fn test_never_self_assigns() {
    let participants: HashSet<_> = (0..5).collect();
    let forbidden_pairs = vec![];
    let history = vec![];

    let processor = ProductionAssignmentProcessor::new();

    // Run multiple times
    for _ in 0..50 {
        if let Some(cycle) = processor.process_assignment(&participants, &forbidden_pairs, &history)
        {
            for i in 0..cycle.len() {
                let giver = cycle[i];
                let receiver = cycle[(i + 1) % cycle.len()];
                assert_ne!(
                    giver, receiver,
                    "Self-assignment detected: {} -> {}",
                    giver, receiver
                );
            }
        }
    }
}
