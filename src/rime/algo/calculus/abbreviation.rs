use std::any::Any;
use std::sync::Arc;

use regex::Regex;

use crate::rime::algo::calculus::calculation::Calculation;
use crate::rime::algo::calculus::{transformation, K_ABBREVIATION_PENALTY};
use crate::rime::algo::spelling::{Spelling, SpellingType};

pub struct Abbreviation {
    pattern: Regex,
    replacement: String,
}

impl Abbreviation {
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

impl Calculation for Abbreviation {
    fn apply(&self, spelling: Option<&mut Spelling>) -> bool {
        let Some(spelling) = spelling else {
            return false;
        };

        let result = transformation::apply(spelling, &self.pattern, &self.replacement);
        if result {
            spelling.properties.type_ = SpellingType::Abbreviation;
            spelling.properties.credibility += K_ABBREVIATION_PENALTY;
        }
        result
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
