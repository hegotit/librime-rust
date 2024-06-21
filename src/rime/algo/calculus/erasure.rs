use std::any::Any;

use regex::Regex;

use crate::rime::algo::calculus::calculation::Calculation;
use crate::rime::algo::spelling::Spelling;

pub(crate) struct Erasure {
    pattern: Regex,
}

impl Erasure {
    pub(crate) fn parse(args: Vec<String>) -> Option<Box<dyn Calculation>> {
        if args.len() < 2 {
            return None;
        }
        let pattern = &args[1];
        if pattern.is_empty() {
            return None;
        }
        Some(Box::new(Self {
            pattern: Regex::new(pattern).unwrap(),
        }))
    }
}

impl Calculation for Erasure {
    fn apply(&self, spelling: Option<&mut Spelling>) -> bool {
        let Some(spelling) = spelling else {
            return false;
        };

        if spelling.str.is_empty() {
            return false;
        }
        if self.pattern.is_match(&spelling.str) {
            spelling.str.clear();
            true
        } else {
            false
        }
    }

    fn addition(&self) -> bool {
        false
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn type_name(&self) -> String {
        String::from("erase")
    }
}
