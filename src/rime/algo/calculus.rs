pub(crate) mod abbreviation;
pub mod calculation;
pub(crate) mod derivation;
pub(crate) mod erasure;
pub(crate) mod fuzzing;
pub(crate) mod transformation;
pub(crate) mod transliteration;

pub(crate) const ABBREVIATION_PENALTY: f64 = -std::f64::consts::LN_2;
pub(crate) const FUZZY_SPELLING_PENALTY: f64 = -std::f64::consts::LN_2;
