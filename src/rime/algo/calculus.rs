mod abbreviation;
pub(crate) mod calculation;
mod derivation;
mod erasure;
mod fuzzing;
mod transformation;
mod transliteration;

pub const K_ABBREVIATION_PENALTY: f64 = -std::f64::consts::LN_2;
pub const K_FUZZY_SPELLING_PENALTY: f64 = -std::f64::consts::LN_2;
