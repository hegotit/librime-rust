use std::any::Any;

use regex::Regex;

use crate::rime::algo::calculus::calculation::Calculation;
use crate::rime::algo::calculus::{transformation, ABBREVIATION_PENALTY};
use crate::rime::algo::spelling::{Spelling, SpellingType};

pub(crate) struct Abbreviation {
    pattern: Regex,
    replacement: String,
}

impl Abbreviation {
    pub(crate) fn parse(args: Vec<String>) -> Option<Box<dyn Calculation>> {
        if args.len() < 3 {
            return None;
        }
        let left = &args[1];
        let right = &args[2];
        if left.is_empty() {
            return None;
        }
        Some(Box::new(Self {
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
            spelling.properties.credibility += ABBREVIATION_PENALTY;
        }
        result
    }

    fn deletion(&self) -> bool {
        false
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn type_name(&self) -> String {
        String::from("abbrev")
    }
}
