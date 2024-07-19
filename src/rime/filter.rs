use crate::rime::{candidate::CandidateList, translation::Translation};
use std::sync::Arc;

pub(crate) trait Filter {
    fn apply(&self, translation: Arc<dyn Translation>, candidates: &mut CandidateList);
}
