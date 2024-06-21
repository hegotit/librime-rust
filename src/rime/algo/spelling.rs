use std::cmp::Ordering;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub(crate) enum SpellingType {
    NormalSpelling,
    FuzzySpelling,
    Abbreviation,
    Completion,
    AmbiguousSpelling,
    InvalidSpelling,
}

#[derive(Debug, Clone)]
pub(crate) struct SpellingProperties {
    pub(crate) type_: SpellingType,
    end_pos: usize,
    pub(crate) credibility: f64,
    pub(crate) tips: String,
}

impl Default for SpellingProperties {
    fn default() -> Self {
        Self {
            type_: SpellingType::NormalSpelling,
            end_pos: 0,
            credibility: 0.0,
            tips: String::new(),
        }
    }
}

#[derive(Default, Debug, Clone)]
pub(crate) struct Spelling {
    pub(crate) str: String,
    pub(crate) properties: SpellingProperties,
}

impl Spelling {
    pub(crate) fn new(str: &str) -> Self {
        Spelling {
            str: str.to_string(),
            ..Default::default()
        }
    }
}

impl PartialEq<Self> for Spelling {
    fn eq(&self, other: &Self) -> bool {
        self.str == other.str
    }
}

impl Eq for Spelling {}

impl PartialOrd for Spelling {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.str.cmp(&other.str))
    }
}

impl Ord for Spelling {
    fn cmp(&self, other: &Self) -> Ordering {
        self.str.cmp(&other.str)
    }
}

fn main() {
    let spelling1 = Spelling::new("example");
    let spelling2 = Spelling::new("example");

    if spelling1 == spelling2 {
        println!("spelling1 and spelling2 are equal");
    }

    if spelling1 < spelling2 {
        println!("spelling1 is less than spelling2");
    } else {
        println!("spelling1 is not less than spelling2");
    }
}
