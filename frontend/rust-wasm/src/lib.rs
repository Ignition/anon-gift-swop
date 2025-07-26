use js_sys::Array;
use std::collections::{HashMap, HashSet};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsValue;

use crate::forbidden_pair::ForbiddenPair;
use assignment::{Assignment, AssignmentPair};
use default_map::DefaultMap;
use production_optimized::ProductionAssignmentProcessor;
use threshold_selector::ThresholdSelector;
use total_weight::TotalWeight;
use walker::SolutionWalker;

mod assignment;
mod default_map;
mod forbidden_pair;
mod mcmc_sampler;
mod threshold_selector;
mod total_weight;
mod url_encoding;
mod walker;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod production_tests;

#[cfg(test)]
mod property_tests;

mod production_optimized;

// Re-export URL encoding functions
pub use url_encoding::{decode_message_v2, decode_state_v2, encode_message_v2, encode_state_v2};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = Math)]
    fn random() -> f64;
}

// Global production processor instance
use std::sync::OnceLock;
static PRODUCTION_PROCESSOR: OnceLock<ProductionAssignmentProcessor> = OnceLock::new();

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[must_use]
#[wasm_bindgen]
#[allow(clippy::cast_precision_loss, clippy::needless_pass_by_value)] // WASM interop
pub fn process_assignment(
    selection: JsValue,
    forbidden_pairs: JsValue,
    history: JsValue,
) -> JsValue {
    let selection_set = to_rust_selection(&selection);
    let forbidden_pairs = to_rust_forbidden_pairs(&forbidden_pairs);
    let history_vec = to_rust_history(&history);

    // Use production-optimized processor with hybrid algorithm selection, convergence detection, caching, and branch-and-bound
    let processor = PRODUCTION_PROCESSOR.get_or_init(ProductionAssignmentProcessor::new);
    match processor.process_assignment(&selection_set, &forbidden_pairs, &history_vec) {
        Some(cycle) => JsValue::from(
            cycle
                .iter()
                .map(|&number| JsValue::from_f64(number as f64))
                .collect::<Array>(),
        ),
        None => JsValue::UNDEFINED,
    }
}

// Internal function using walker algorithm
fn process_assignment_internal(
    selection_set: &HashSet<usize>,
    forbidden_pairs: &[ForbiddenPair],
    history_vec: &[Assignment],
) -> JsValue {
    // Single participant case is impossible (can't give to themselves)
    if selection_set.len() == 1 {
        return JsValue::UNDEFINED;
    }

    let mut buffer: Vec<_> = selection_set.iter().copied().collect();
    let weights = weights_for(selection_set, forbidden_pairs, history_vec);

    // Get the total weight of all possible valid selections
    let mut total_weight_walker = TotalWeight::default();
    walker::apply_walker(&weights, &mut buffer, &mut total_weight_walker);
    let total_weight = total_weight_walker.result();

    if total_weight <= 0.0 {
        return JsValue::UNDEFINED;
    }

    // Randomly select a given threshold for selection
    let r = random();
    let weight_threshold: f64 = r * total_weight;

    // Walk again to extract the selection
    let mut threshold_selector_walker = ThresholdSelector::new(weight_threshold);
    walker::apply_walker(&weights, &mut buffer, &mut threshold_selector_walker);

    match threshold_selector_walker.result() {
        None => JsValue::UNDEFINED,
        Some(selection) => {
            #[allow(clippy::cast_precision_loss)] // WASM requires f64 for JS interop
            let js_array: Array = selection
                .iter()
                .map(|&number| JsValue::from_f64(number as f64))
                .collect();
            JsValue::from(js_array)
        }
    }
}

// Export original algorithm for comparison/debugging
#[must_use]
#[wasm_bindgen]
#[allow(clippy::cast_precision_loss, clippy::needless_pass_by_value)] // WASM interop
pub fn process_assignment_original(
    selection: JsValue,
    forbidden_pairs: JsValue,
    history: JsValue,
) -> JsValue {
    let selection_set = to_rust_selection(&selection);
    let forbidden_pairs = to_rust_forbidden_pairs(&forbidden_pairs);
    let history_vec = to_rust_history(&history);

    process_assignment_internal(&selection_set, &forbidden_pairs, &history_vec)
}

// Cache management functions for monitoring and optimization
#[must_use]
#[wasm_bindgen]
pub fn get_cache_stats() -> JsValue {
    let processor = PRODUCTION_PROCESSOR.get_or_init(ProductionAssignmentProcessor::new);
    let stats = processor.cache_stats();

    let obj = js_sys::Object::new();
    js_sys::Reflect::set(
        &obj,
        &"edgeHitRate".into(),
        &(stats.edge_hit_rate * 100.0).into(),
    )
    .unwrap();
    js_sys::Reflect::set(
        &obj,
        &"cycleHitRate".into(),
        &(stats.cycle_hit_rate * 100.0).into(),
    )
    .unwrap();
    js_sys::Reflect::set(
        &obj,
        &"edgeCacheSize".into(),
        &(stats.edge_cache_size as f64).into(),
    )
    .unwrap();
    js_sys::Reflect::set(
        &obj,
        &"cycleCacheSize".into(),
        &(stats.cycle_cache_size as f64).into(),
    )
    .unwrap();
    js_sys::Reflect::set(
        &obj,
        &"totalEdgeLookups".into(),
        &(stats.total_edge_lookups as f64).into(),
    )
    .unwrap();
    js_sys::Reflect::set(
        &obj,
        &"totalCycleLookups".into(),
        &(stats.total_cycle_lookups as f64).into(),
    )
    .unwrap();

    JsValue::from(obj)
}

#[wasm_bindgen]
pub fn clear_cache() {
    let processor = PRODUCTION_PROCESSOR.get_or_init(ProductionAssignmentProcessor::new);
    processor.clear_cache();
}

fn weights_for(
    participants: &HashSet<usize>,
    forbidden_pairs: &[ForbiddenPair],
    previous_assignments: &[Assignment],
) -> DefaultMap<AssignmentPair, f64> {
    // useful set for filtering out non-participants
    let participant_set: HashSet<_> = participants.iter().copied().collect();

    // Default weight will be 1.0
    // 0.0 will be an impossible assignment
    // Values between are due to disincentive based on previous assignments
    let default_weight = 1.0;
    let mut weights = HashMap::new();

    // forbidden pairs
    for fp in forbidden_pairs {
        // ignore entries which are not part of the current set of participants
        if fp.invalid(&participant_set) {
            continue;
        }

        // both giving and receiving assignments are invalid
        weights.insert(
            AssignmentPair {
                giver: fp.member_1,
                receiver: fp.member_2,
            },
            0.0,
        );
        weights.insert(
            AssignmentPair {
                giver: fp.member_2,
                receiver: fp.member_1,
            },
            0.0,
        );
    }

    // disincentive for recent history - stronger penalties
    for &giver in participants {
        // History penalty structure: 0.5, 0.25, 0.125, ... (geometric decay)
        let disincentives = (0..).map(|i| 0.5 / 2.0_f64.powi(i));
        let receiver_penalties = previous_assignments
            .iter()
            // selection that the `giver` was involved in, get the receiver
            .filter_map(|selection| selection.receiver_of(giver))
            // check receiver is relevant to the current participants
            .filter(|receiver| participant_set.contains(receiver))
            // historic assignment have decreasing disincentives
            .zip(disincentives);

        for (receiver, penalty) in receiver_penalties {
            weights
                .entry(AssignmentPair { giver, receiver })
                .and_modify(|w| *w = (*w - penalty).max(0.001)) // Prevent negative/zero weights
                .or_insert((default_weight - penalty).max(0.001));
        }
    }

    DefaultMap {
        map: weights,
        default_value: default_weight,
    }
}

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)] // WASM JS->Rust conversion
fn to_rust_history(history: &JsValue) -> Vec<Assignment> {
    history
        .dyn_ref::<Array>()
        .unwrap()
        .to_vec()
        .iter()
        .filter_map(|v| {
            let arr = v.dyn_ref::<Array>()?;
            let value: Vec<_> = arr
                .to_vec()
                .iter()
                .filter_map(wasm_bindgen::JsValue::as_f64)
                .map(|v| v as usize)
                .collect();
            Some(Assignment { value })
        })
        .collect()
}

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)] // WASM JS->Rust conversion
fn to_rust_forbidden_pairs(forbidden_pairs: &JsValue) -> Vec<ForbiddenPair> {
    forbidden_pairs
        .dyn_ref::<Array>()
        .unwrap()
        .to_vec()
        .iter()
        .filter_map(|v| {
            let arr = v.dyn_ref::<Array>()?;
            let first = arr.get(0).as_f64()? as usize;
            let second = arr.get(1).as_f64()? as usize;
            if first < second {
                Some(ForbiddenPair {
                    member_1: first,
                    member_2: second,
                })
            } else {
                Some(ForbiddenPair {
                    member_1: second,
                    member_2: first,
                })
            }
        })
        .collect()
}

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)] // WASM JS->Rust conversion
fn to_rust_selection(selection: &JsValue) -> HashSet<usize> {
    selection
        .dyn_ref::<Array>()
        .unwrap()
        .to_vec()
        .iter()
        .filter_map(wasm_bindgen::JsValue::as_f64)
        .map(|v| v as usize)
        .collect()
}
