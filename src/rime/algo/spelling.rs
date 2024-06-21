use std::cmp::Ordering;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum SpellingType {
    Normal,
    Fuzzy,
    Abbreviation,
    Completion,
    Ambiguous,
    Invalid,
}

impl Default for SpellingType {
    fn default() -> Self {
        SpellingType::Normal
    }
}

impl TryFrom<i32> for SpellingType {
    type Error = &'static str;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(SpellingType::Normal),
            1 => Ok(SpellingType::Fuzzy),
            2 => Ok(SpellingType::Abbreviation),
            3 => Ok(SpellingType::Completion),
            4 => Ok(SpellingType::Ambiguous),
            5 => Ok(SpellingType::Invalid),
            _ => Err("Invalid integer for SpellingType"),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct SpellingProperties {
    pub type_: SpellingType,
    pub end_pos: usize,
    pub credibility: f64,
    pub(crate) tips: String,
}

impl SpellingProperties {
    pub(crate) fn new(type_: SpellingType, end_pos: usize, credibility: f64, tips: String) -> Self {
        Self {
            type_,
            end_pos,
            credibility,
            tips,
        }
    }
}

#[derive(Default, Debug, Clone)]
pub struct Spelling {
    pub str: String,
    pub properties: SpellingProperties,
}

impl Spelling {
    pub fn new(str: &str) -> Self {
        Self {
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

#[test]
fn test() {
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
