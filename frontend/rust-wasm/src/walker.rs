use crate::assignment::AssignmentPair;
use crate::default_map::DefaultMap;
pub(crate) trait SolutionWalker {
    type Result;
    fn visit(&mut self, selection: &[usize], weight: f64) -> bool;
    fn result(&self) -> Self::Result;
}

fn walk_solutions<T>(
    current: usize,
    current_weight: f64,
    weights: &DefaultMap<AssignmentPair, f64>,
    buffer: &mut Vec<usize>,
    visitor: &mut T,
) -> bool
where
    T: SolutionWalker,
{
    let giver = buffer[current];
    if current == buffer.len() - 1 {
        let receiver = buffer[0];
        let weight = weights.get(&AssignmentPair { giver, receiver });

        // if weight is 0.0 then no solution involving: giver --> receiver
        if weight == 0.0 {
            return false;
        }

        visitor.visit(buffer, weight * current_weight)
    } else {
        let frontier = current + 1;
        for next in frontier..buffer.len() {
            let receiver = buffer[next];
            let weight = weights.get(&AssignmentPair { giver, receiver });
            // if weight is 0.0 then no solution involving: giver --> receiver
            if weight == 0.0 {
                continue;
            }
            // move next to be current
            buffer.swap(frontier, next);
            if walk_solutions(frontier, current_weight * weight, weights, buffer, visitor) {
                return true;
            }
            // restore the order of the buffer
            buffer.swap(frontier, next);
        }
        false
    }
}

#[allow(clippy::module_name_repetitions)] // Walker is the core algorithm
pub fn apply_walker<T>(
    weights: &DefaultMap<AssignmentPair, f64>,
    buffer: &mut Vec<usize>,
    walker: &mut T,
) where
    T: SolutionWalker,
{
    walk_solutions(0, 1.0, weights, buffer, walker);
}
