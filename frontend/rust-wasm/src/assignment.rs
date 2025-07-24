#[derive(Debug, Eq, Hash, PartialEq)]
#[allow(clippy::module_name_repetitions)] // Clear naming for WASM export
pub struct AssignmentPair {
    pub(crate) giver: usize,
    pub(crate) receiver: usize,
}

#[derive(Default, Clone)]
pub struct Assignment {
    pub(crate) value: Vec<usize>,
}

impl Assignment {
    pub fn receiver_of(&self, i: usize) -> Option<usize> {
        if let Some(pos) = self.value.iter().position(|&x| x == i) {
            let next_pos = (pos + 1) % self.value.len();
            Some(self.value[next_pos])
        } else {
            None
        }
    }
}
