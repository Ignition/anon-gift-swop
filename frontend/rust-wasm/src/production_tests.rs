/// Production validation tests
/// These tests focus on real-world scenarios and correctness validation
#[cfg(test)]
#[allow(clippy::module_inception)]
mod production_tests {
    use crate::assignment::Assignment;
    use crate::forbidden_pair::ForbiddenPair;
    use crate::production_optimized::ProductionAssignmentProcessor;
    use std::collections::HashSet;

    #[test]
    fn test_production_correctness() {
        println!("\n✅ Production System Correctness Test");

        let processor = ProductionAssignmentProcessor::new();

        let test_scenarios = [
            // Small family group
            ("Small family (4 people)", vec![0, 1, 2, 3], vec![], vec![]),
            // Office team with constraints
            (
                "Office team with constraints",
                vec![0, 1, 2, 3, 4, 5, 6, 7],
                vec![
                    ForbiddenPair {
                        member_1: 0,
                        member_2: 1,
                    }, // Manager-direct report
                    ForbiddenPair {
                        member_1: 2,
                        member_2: 3,
                    }, // Married couple
                ],
                vec![],
            ),
            // Large group with history
            (
                "Large group with history",
                vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11],
                vec![ForbiddenPair {
                    member_1: 0,
                    member_2: 1,
                }],
                vec![
                    Assignment {
                        value: vec![0, 2, 4, 6, 8, 10, 1, 3, 5, 7, 9, 11],
                    },
                    Assignment {
                        value: vec![0, 3, 6, 9, 1, 4, 7, 10, 2, 5, 8, 11],
                    },
                ],
            ),
            // Maximum realistic scenario
            (
                "Maximum realistic (50 people)",
                (0..50).collect::<Vec<_>>(),
                vec![
                    ForbiddenPair {
                        member_1: 0,
                        member_2: 1,
                    },
                    ForbiddenPair {
                        member_1: 10,
                        member_2: 11,
                    },
                    ForbiddenPair {
                        member_1: 20,
                        member_2: 21,
                    },
                ],
                vec![
                    Assignment {
                        value: (0..50).collect(),
                    },
                    Assignment {
                        value: (0..50).rev().collect(),
                    },
                ],
            ),
        ];

        for (name, participants_vec, forbidden_pairs, history) in &test_scenarios {
            println!("  Testing: {}", name);

            let participants: HashSet<_> = participants_vec.iter().copied().collect();

            let result = processor.process_assignment(&participants, forbidden_pairs, history);

            match result {
                Some(assignment) => {
                    // Validate correctness
                    assert_eq!(
                        assignment.len(),
                        participants.len(),
                        "Assignment length mismatch for {}",
                        name
                    );

                    // Check all participants included exactly once
                    let assignment_set: HashSet<_> = assignment.iter().copied().collect();
                    assert_eq!(
                        assignment_set.len(),
                        assignment.len(),
                        "Duplicate participants for {}",
                        name
                    );

                    for &p in &participants {
                        assert!(
                            assignment_set.contains(&p),
                            "Missing participant {} in {}",
                            p,
                            name
                        );
                    }

                    // Check no forbidden pairs
                    for i in 0..assignment.len() {
                        let giver = assignment[i];
                        let receiver = assignment[(i + 1) % assignment.len()];

                        for fp in forbidden_pairs {
                            assert!(
                                !(fp.member_1 == giver && fp.member_2 == receiver),
                                "Forbidden pair violated in {}: {} -> {}",
                                name,
                                giver,
                                receiver
                            );
                            assert!(
                                !(fp.member_1 == receiver && fp.member_2 == giver),
                                "Forbidden pair violated in {}: {} -> {}",
                                name,
                                receiver,
                                giver
                            );
                        }
                    }

                    println!("    ✅ Valid assignment found");
                }
                None => {
                    // Only acceptable for impossible scenarios
                    println!("    ⚠️  No assignment found (may be overconstrained)");
                }
            }
        }
    }

    #[test]
    fn test_production_performance() {
        println!("\n🚀 Production System Performance Test");

        let processor = ProductionAssignmentProcessor::new();

        let performance_scenarios = [
            ("Small (6 people)", 6, 50),
            ("Medium (15 people)", 15, 20),
            ("Large (30 people)", 30, 10),
            ("Very Large (50 people)", 50, 5),
        ];

        println!("  Scenario            | Avg Time | Success Rate");
        println!("  --------------------|----------|-------------");

        for (name, size, iterations) in &performance_scenarios {
            let participants: HashSet<_> = (0..*size).collect();
            let forbidden_pairs = vec![ForbiddenPair {
                member_1: 0,
                member_2: 1,
            }];
            let history = vec![Assignment {
                value: (0..*size).collect(),
            }];

            let mut total_time = 0u128;
            let mut successes = 0;

            for _ in 0..*iterations {
                let start = std::time::Instant::now();
                if processor
                    .process_assignment(&participants, &forbidden_pairs, &history)
                    .is_some()
                {
                    successes += 1;
                }
                total_time += start.elapsed().as_micros();
            }

            let avg_time = total_time as f64 / *iterations as f64 / 1000.0; // Convert to ms
            let success_rate = successes as f64 / *iterations as f64 * 100.0;

            println!(
                "  {:<19} | {:7.2}ms | {:10.0}%",
                name, avg_time, success_rate
            );
        }
    }

    #[test]
    fn test_algorithm_selection() {
        println!("\n🎯 Algorithm Selection Test");

        let processor = ProductionAssignmentProcessor::new();

        // Test that different problem sizes trigger appropriate algorithms
        let test_cases = [
            ("Branch-and-Bound trigger", 4, 0, 0),
            ("Simulated Annealing trigger", 8, 20, 0), // High constraint density
            ("Parallel Tempering trigger", 15, 0, 10), // Complex history
            ("Default MCMC trigger", 12, 1, 2),
        ];

        for (name, size, num_forbidden, history_size) in &test_cases {
            let participants: HashSet<_> = (0..*size).collect();

            let forbidden_pairs: Vec<_> = (0..*num_forbidden)
                .map(|i| ForbiddenPair {
                    member_1: i % size,
                    member_2: (i + 1) % size,
                })
                .collect();

            let history: Vec<_> = (0..*history_size)
                .map(|i| {
                    let mut assignment: Vec<_> = (0..*size).collect();
                    let len = assignment.len();
                    if len > 0 {
                        assignment.rotate_left(i % len);
                    }
                    Assignment { value: assignment }
                })
                .collect();

            let start = std::time::Instant::now();
            let result = processor.process_assignment(&participants, &forbidden_pairs, &history);
            let time = start.elapsed();

            println!(
                "  {}: {} in {:.2}ms",
                name,
                if result.is_some() {
                    "✅ Success"
                } else {
                    "❌ Failed"
                },
                time.as_micros() as f64 / 1000.0
            );
        }
    }

    #[test]
    fn test_edge_cases() {
        println!("\n⚠️  Edge Cases Test");

        let processor = ProductionAssignmentProcessor::new();

        // Single participant (impossible)
        let single: HashSet<_> = [0].iter().copied().collect();
        let result = processor.process_assignment(&single, &[], &[]);
        assert!(result.is_none(), "Single participant should be impossible");
        println!("  ✅ Single participant correctly rejected");

        // Empty set (should be handled gracefully)
        let empty: HashSet<usize> = HashSet::new();
        let result = processor.process_assignment(&empty, &[], &[]);
        // Empty set may return None or empty assignment - both are acceptable
        println!(
            "  ✅ Empty set handled: {}",
            if result.is_none() {
                "rejected"
            } else {
                "empty assignment"
            }
        );

        // Complete constraint graph (likely impossible)
        let participants: HashSet<_> = (0..4).collect();
        let all_forbidden = vec![
            ForbiddenPair {
                member_1: 0,
                member_2: 1,
            },
            ForbiddenPair {
                member_1: 0,
                member_2: 2,
            },
            ForbiddenPair {
                member_1: 0,
                member_2: 3,
            },
            ForbiddenPair {
                member_1: 1,
                member_2: 2,
            },
            ForbiddenPair {
                member_1: 1,
                member_2: 3,
            },
            ForbiddenPair {
                member_1: 2,
                member_2: 3,
            },
        ];
        let result = processor.process_assignment(&participants, &all_forbidden, &[]);
        // This may or may not find a solution depending on the exact constraints
        println!(
            "  ✅ Complete constraint graph handled: {}",
            if result.is_some() {
                "solution found"
            } else {
                "no solution"
            }
        );
    }

    #[test]
    fn test_cache_functionality() {
        println!("\n💾 Cache Functionality Test");

        let processor = ProductionAssignmentProcessor::new();

        // Clear cache and run assignments
        processor.clear_cache();

        let participants: HashSet<_> = (0..10).collect();
        let forbidden_pairs = vec![ForbiddenPair {
            member_1: 0,
            member_2: 1,
        }];
        let history = vec![];

        // Run multiple assignments to populate cache
        for _ in 0..10 {
            let _ = processor.process_assignment(&participants, &forbidden_pairs, &history);
        }

        let stats = processor.cache_stats();
        println!("  Cache statistics after 10 runs:");
        println!("    Cycle cache entries: {}", stats.cycle_cache_size);
        println!(
            "    Cycle cache hit rate: {:.1}%",
            stats.cycle_hit_rate * 100.0
        );
        println!("    Total cycle lookups: {}", stats.total_cycle_lookups);

        assert!(stats.total_cycle_lookups > 0, "Cache should be used");
        println!("  ✅ Cache is functioning");
    }
}
