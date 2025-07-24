use std::collections::HashSet;
#[derive(Clone, Debug)]
pub struct ForbiddenPair {
    pub(crate) member_1: usize,
    pub(crate) member_2: usize,
}

impl ForbiddenPair {
    pub fn valid(&self, valid_set: &HashSet<usize>) -> bool {
        valid_set.contains(&self.member_1) && valid_set.contains(&self.member_2)
    }

    pub fn invalid(&self, valid_set: &HashSet<usize>) -> bool {
        !self.valid(valid_set)
    }
}
