use crate::walker::SolutionWalker;

#[derive(Default)]
pub struct TotalWeight {
    total_weight: f64,
}

impl SolutionWalker for TotalWeight {
    type Result = f64;

    fn visit(&mut self, _selection: &[usize], weight: f64) -> bool {
        self.total_weight += weight;
        false
    }

    fn result(&self) -> Self::Result {
        self.total_weight
    }
}
