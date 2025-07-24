use crate::walker::SolutionWalker;

pub struct ThresholdSelector {
    total_weight: f64,
    threshold: f64,
    selection: Option<Vec<usize>>,
}

impl SolutionWalker for ThresholdSelector {
    type Result = Option<Vec<usize>>;

    fn visit(&mut self, selection: &[usize], weight: f64) -> bool {
        self.total_weight += weight;
        if self.total_weight >= self.threshold {
            self.selection = Some(selection.to_vec());
            true
        } else {
            false
        }
    }

    fn result(&self) -> Self::Result {
        //TODO: don't copy
        self.selection.clone()
    }
}

impl ThresholdSelector {
    pub(crate) fn new(threshold: f64) -> Self {
        Self {
            total_weight: 0.0,
            threshold,
            selection: None,
        }
    }
}
