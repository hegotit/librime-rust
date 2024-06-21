use regex::Regex;
use std::any::Any;
use std::sync::Arc;

use crate::rime::algo::calculus::calculation::Calculation;
use crate::rime::algo::calculus::transformation;
use crate::rime::algo::calculus::K_FUZZY_SPELLING_PENALTY;
use crate::rime::algo::spelling::{Spelling, SpellingType};

pub struct Fuzzing {
    pattern: Regex,
    replacement: String,
}

impl Fuzzing {
    pub fn parse(args: Vec<String>) -> Option<Arc<dyn Calculation>> {
        if args.len() < 3 {
            return None;
        }
        let left = &args[1];
        let right = &args[2];
        if left.is_empty() {
            return None;
        }
        Some(Arc::new(Self {
            pattern: Regex::new(left).unwrap(),
            replacement: right.to_string(),
        }))
    }
}

impl Calculation for Fuzzing {
    fn apply(&self, spelling: Option<&mut Spelling>) -> bool {
        let Some(spelling) = spelling else {
            return false;
        };
        let result = transformation::apply(spelling, &self.pattern, &self.replacement);
        if result {
            spelling.properties.type_ = SpellingType::FuzzySpelling;
            spelling.properties.credibility += K_FUZZY_SPELLING_PENALTY;
        }
        result
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
