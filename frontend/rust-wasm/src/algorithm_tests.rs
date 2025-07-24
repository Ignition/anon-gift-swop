use super::*;
use std::collections::HashSet;
use wasm_bindgen::JsValue;
use js_sys::Array;

// Helper function to convert Rust data to JsValue for process_assignment
fn create_js_selection(selection: &[usize]) -> JsValue {
    let array = Array::new();
    for &item in selection {
        array.push(&JsValue::from_f64(item as f64));
    }
    JsValue::from(array)
}

fn create_js_forbidden_pairs(pairs: &[(usize, usize)]) -> JsValue {
    let array = Array::new();
    for &(a, b) in pairs {
        let pair_array = Array::new();
        pair_array.push(&JsValue::from_f64(a as f64));
        pair_array.push(&JsValue::from_f64(b as f64));
        array.push(&pair_array);
    }
    JsValue::from(array)
}

fn create_js_history(history: &[Vec<usize>]) -> JsValue {
    let array = Array::new();
    for assignment in history {
        let assignment_array = Array::new();
        for &item in assignment {
            assignment_array.push(&JsValue::from_f64(item as f64));
        }
        array.push(&assignment_array);
    }
    JsValue::from(array)
}

fn js_to_rust_assignment(js_value: JsValue) -> Option<Vec<usize>> {
    if js_value.is_undefined() || js_value.is_null() {
        return None;
    }
    
    let array = js_value.dyn_ref::<Array>()?;
    let result: Vec<usize> = array
        .to_vec()
        .iter()
        .filter_map(|v| v.as_f64().map(|f| f as usize))
        .collect();
    
    Some(result)
}

#[test]
fn test_basic_assignment() {
    let selection = vec![0, 1, 2];
    let js_selection = create_js_selection(&selection);
    let js_forbidden = create_js_forbidden_pairs(&[]);
    let js_history = create_js_history(&[]);
    
    let result = process_assignment(js_selection, js_forbidden, js_history);
    let assignment = js_to_rust_assignment(result);
    
    assert!(assignment.is_some(), "Should create a valid assignment");
    let assignment = assignment.unwrap();
    
    // Verify it's a valid cycle
    assert_eq!(assignment.len(), selection.len());
    
    // Check no self-assignments
    for (giver, &receiver) in assignment.iter().enumerate() {
        if selection.contains(&giver) {
            assert_ne!(giver, receiver, "No one should give to themselves");
        }
    }
    
    // Check everyone receives exactly once
    let mut receivers = HashSet::new();
    for &participant in &selection {
        receivers.insert(assignment[participant]);
    }
    assert_eq!(receivers.len(), selection.len(), "Everyone should receive exactly once");
}

#[test]
fn test_respects_forbidden_pairs() {
    let selection = vec![0, 1, 2, 3];
    let forbidden_pairs = vec![(0, 1), (2, 3)];
    
    // Run multiple times to ensure constraints are always respected
    for _ in 0..20 {
        let js_selection = create_js_selection(&selection);
        let js_forbidden = create_js_forbidden_pairs(&forbidden_pairs);
        let js_history = create_js_history(&[]);
        
        let result = process_assignment(js_selection, js_forbidden, js_history);
        
        if let Some(assignment) = js_to_rust_assignment(result) {
            assert_ne!(assignment[0], 1, "0 should not give to 1");
            assert_ne!(assignment[1], 0, "1 should not give to 0");
            assert_ne!(assignment[2], 3, "2 should not give to 3");
            assert_ne!(assignment[3], 2, "3 should not give to 2");
        }
    }
}

#[test]
fn test_impossible_assignment() {
    let selection = vec![0, 1, 2];
    // Make it impossible - everyone forbidden from everyone else
    let forbidden_pairs = vec![
        (0, 1), (0, 2),
        (1, 0), (1, 2),
        (2, 0), (2, 1),
    ];
    
    let js_selection = create_js_selection(&selection);
    let js_forbidden = create_js_forbidden_pairs(&forbidden_pairs);
    let js_history = create_js_history(&[]);
    
    let result = process_assignment(js_selection, js_forbidden, js_history);
    
    assert!(
        result.is_undefined() || result.is_null(),
        "Should return undefined for impossible case"
    );
}

#[test]
fn test_single_participant() {
    let selection = vec![0];
    
    let js_selection = create_js_selection(&selection);
    let js_forbidden = create_js_forbidden_pairs(&[]);
    let js_history = create_js_history(&[]);
    
    let result = process_assignment(js_selection, js_forbidden, js_history);
    
    assert!(
        result.is_undefined() || result.is_null(),
        "Single participant cannot form a valid cycle"
    );
}

#[test]
fn test_two_participants() {
    let selection = vec![0, 1];
    
    let js_selection = create_js_selection(&selection);
    let js_forbidden = create_js_forbidden_pairs(&[]);
    let js_history = create_js_history(&[]);
    
    let result = process_assignment(js_selection, js_forbidden, js_history);
    let assignment = js_to_rust_assignment(result);
    
    assert!(assignment.is_some(), "Two participants should be able to exchange");
    
    if let Some(assignment) = assignment {
        assert_eq!(assignment[0], 1, "0 should give to 1");
        assert_eq!(assignment[1], 0, "1 should give to 0");
    }
}

#[test]
fn test_considers_history() {
    let selection = vec![0, 1, 2];
    let history = vec![vec![1, 2, 0]]; // Previous: 0->1, 1->2, 2->0
    
    // Track how often we get the historical pattern
    let mut historical_count = 0;
    let runs = 100;
    
    for _ in 0..runs {
        let js_selection = create_js_selection(&selection);
        let js_forbidden = create_js_forbidden_pairs(&[]);
        let js_history = create_js_history(&history);
        
        let result = process_assignment(js_selection, js_forbidden, js_history);
        
        if let Some(assignment) = js_to_rust_assignment(result) {
            if assignment[0] == 1 && assignment[1] == 2 && assignment[2] == 0 {
                historical_count += 1;
            }
        }
    }
    
    // Historical pattern should appear less frequently due to penalty
    assert!(
        historical_count < runs / 3,
        "Historical pattern appeared {} times out of {}, should be less frequent",
        historical_count,
        runs
    );
}

#[test]
fn test_large_group() {
    let selection: Vec<usize> = (0..20).collect();
    
    let js_selection = create_js_selection(&selection);
    let js_forbidden = create_js_forbidden_pairs(&[]);
    let js_history = create_js_history(&[]);
    
    let result = process_assignment(js_selection, js_forbidden, js_history);
    let assignment = js_to_rust_assignment(result);
    
    assert!(assignment.is_some(), "Should handle large groups");
    
    if let Some(assignment) = assignment {
        // Verify all participants are assigned
        let mut receivers = HashSet::new();
        for i in 0..20 {
            receivers.insert(assignment[i]);
        }
        assert_eq!(receivers.len(), 20, "All participants should receive exactly once");
    }
}

#[test]
fn test_non_contiguous_ids() {
    let selection = vec![1, 5, 10, 15];
    
    let js_selection = create_js_selection(&selection);
    let js_forbidden = create_js_forbidden_pairs(&[]);
    let js_history = create_js_history(&[]);
    
    let result = process_assignment(js_selection, js_forbidden, js_history);
    let assignment = js_to_rust_assignment(result);
    
    assert!(assignment.is_some(), "Should handle non-contiguous participant IDs");
    
    if let Some(assignment) = assignment {
        // Verify assignments only use valid participant IDs
        for &id in &selection {
            let receiver = assignment[id];
            assert!(
                selection.contains(&receiver),
                "Receiver {} must be a valid participant",
                receiver
            );
            assert_ne!(id, receiver, "No self-assignments");
        }
    }
}

#[test]
fn test_multiple_history_penalty() {
    let selection = vec![0, 1, 2, 3];
    let history = vec![
        vec![1, 2, 3, 0], // Recent: 0->1, 1->2, 2->3, 3->0
        vec![2, 3, 0, 1], // Older: 0->2, 1->3, 2->0, 3->1
    ];
    
    // Count frequency of assignments
    let mut count_0_to_1 = 0;
    let mut count_0_to_2 = 0;
    let runs = 100;
    
    for _ in 0..runs {
        let js_selection = create_js_selection(&selection);
        let js_forbidden = create_js_forbidden_pairs(&[]);
        let js_history = create_js_history(&history);
        
        let result = process_assignment(js_selection, js_forbidden, js_history);
        
        if let Some(assignment) = js_to_rust_assignment(result) {
            if assignment[0] == 1 {
                count_0_to_1 += 1;
            } else if assignment[0] == 2 {
                count_0_to_2 += 1;
            }
        }
    }
    
    // More recent history (0->1) should be penalized more than older (0->2)
    assert!(
        count_0_to_1 < count_0_to_2,
        "Recent assignment 0->1 ({}) should be less frequent than older 0->2 ({})",
        count_0_to_1,
        count_0_to_2
    );
}

#[test]
fn test_stress_complex_constraints() {
    let selection: Vec<usize> = (0..10).collect();
    
    // Each person can't give to their immediate neighbors
    let mut forbidden_pairs = vec![];
    for i in 0..10 {
        forbidden_pairs.push((i, (i + 1) % 10));
        forbidden_pairs.push((i, (i + 9) % 10));
    }
    
    // Should still find valid solutions
    let mut successes = 0;
    for _ in 0..10 {
        let js_selection = create_js_selection(&selection);
        let js_forbidden = create_js_forbidden_pairs(&forbidden_pairs);
        let js_history = create_js_history(&[]);
        
        let result = process_assignment(js_selection, js_forbidden, js_history);
        
        if js_to_rust_assignment(result).is_some() {
            successes += 1;
        }
    }
    
    assert!(successes > 8, "Should successfully find assignments most of the time");
}

#[test]
fn test_maximally_constrained() {
    let selection = vec![0, 1, 2, 3];
    
    // Force the cycle 0->1->2->3->0
    let forbidden_pairs = vec![
        (0, 2), (0, 3),
        (1, 0), (1, 3),
        (2, 0), (2, 1),
        (3, 1), (3, 2),
    ];
    
    // Run multiple times - should always find the same solution
    for _ in 0..5 {
        let js_selection = create_js_selection(&selection);
        let js_forbidden = create_js_forbidden_pairs(&forbidden_pairs);
        let js_history = create_js_history(&[]);
        
        let result = process_assignment(js_selection, js_forbidden, js_history);
        let assignment = js_to_rust_assignment(result);
        
        assert!(assignment.is_some(), "Should find the unique solution");
        
        if let Some(assignment) = assignment {
            assert_eq!(assignment[0], 1);
            assert_eq!(assignment[1], 2);
            assert_eq!(assignment[2], 3);
            assert_eq!(assignment[3], 0);
        }
    }
}

#[test]
fn test_randomness() {
    let selection = vec![0, 1, 2, 3, 4];
    
    let mut results = HashSet::new();
    
    for _ in 0..20 {
        let js_selection = create_js_selection(&selection);
        let js_forbidden = create_js_forbidden_pairs(&[]);
        let js_history = create_js_history(&[]);
        
        let result = process_assignment(js_selection, js_forbidden, js_history);
        
        if let Some(assignment) = js_to_rust_assignment(result) {
            let pattern = assignment.iter()
                .map(|&x| x.to_string())
                .collect::<Vec<_>>()
                .join(",");
            results.insert(pattern);
        }
    }
    
    // Should have at least some variety in results
    assert!(
        results.len() > 1,
        "Algorithm should produce different results, got {} unique patterns",
        results.len()
    );
}

#[test]
fn test_cycle_validity() {
    let selection = vec![0, 1, 2, 3, 4, 5];
    
    for _ in 0..10 {
        let js_selection = create_js_selection(&selection);
        let js_forbidden = create_js_forbidden_pairs(&[]);
        let js_history = create_js_history(&[]);
        
        let result = process_assignment(js_selection, js_forbidden, js_history);
        
        if let Some(assignment) = js_to_rust_assignment(result) {
            // Verify it forms a valid hamiltonian cycle
            let mut visited = HashSet::new();
            let mut current = 0;
            let mut steps = 0;
            
            while !visited.contains(&current) && steps < selection.len() + 1 {
                visited.insert(current);
                current = assignment[current];
                steps += 1;
            }
            
            assert_eq!(visited.len(), selection.len(), "Should visit everyone exactly once");
            assert_eq!(current, 0, "Should return to starting point");
        }
    }
}

#[test]
fn test_couples_scenario() {
    // Simulating couples who shouldn't give to each other
    let selection = vec![0, 1, 2, 3, 4, 5]; // 3 couples
    let forbidden_pairs = vec![
        (0, 1), (1, 0), // Couple 1
        (2, 3), (3, 2), // Couple 2
        (4, 5), (5, 4), // Couple 3
    ];
    
    for _ in 0..10 {
        let js_selection = create_js_selection(&selection);
        let js_forbidden = create_js_forbidden_pairs(&forbidden_pairs);
        let js_history = create_js_history(&[]);
        
        let result = process_assignment(js_selection, js_forbidden, js_history);
        
        if let Some(assignment) = js_to_rust_assignment(result) {
            // Verify couples don't give to each other
            assert_ne!(assignment[0], 1);
            assert_ne!(assignment[1], 0);
            assert_ne!(assignment[2], 3);
            assert_ne!(assignment[3], 2);
            assert_ne!(assignment[4], 5);
            assert_ne!(assignment[5], 4);
        }
    }
}

#[test]
fn test_performance_large_group() {
    use std::time::Instant;
    
    let selection: Vec<usize> = (0..100).collect();
    
    let js_selection = create_js_selection(&selection);
    let js_forbidden = create_js_forbidden_pairs(&[]);
    let js_history = create_js_history(&[]);
    
    let start = Instant::now();
    let result = process_assignment(js_selection, js_forbidden, js_history);
    let duration = start.elapsed();
    
    assert!(
        js_to_rust_assignment(result).is_some(),
        "Should handle 100 participants"
    );
    
    assert!(
        duration.as_millis() < 1000,
        "Should complete within 1 second, took {} ms",
        duration.as_millis()
    );
}