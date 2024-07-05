use crate::rime::algo::SyllableId;
use crate::rime::dict::prism::Prism;
use std::collections::HashMap;

pub(crate) struct Correction {
    distance: usize,
    syllable: SyllableId,
    pub(crate) length: usize,
}

pub(crate) struct Corrections(pub(crate) HashMap<SyllableId, Correction>);

impl Corrections {
    pub(crate) fn new() -> Self {
        Self(HashMap::new())
    }
}

pub(crate) struct Corrector;

impl Corrector {
    pub(crate) fn tolerance_search(
        &self,
        _prism: &Prism,
        _key: &str,
        _results: &mut Corrections,
        _tolerance: usize,
    ) {
    }
}
