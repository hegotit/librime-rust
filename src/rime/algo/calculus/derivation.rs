use std::any::Any;

use regex::Regex;

use crate::rime::algo::calculus::calculation::Calculation;
use crate::rime::algo::calculus::transformation;
use crate::rime::algo::spelling::Spelling;

pub(crate) struct Derivation {
    pattern: Regex,
    replacement: String,
}

impl Derivation {
    pub(crate) fn parse(args: Vec<String>) -> Option<Box<dyn Calculation>> {
        if args.len() < 3 {
            return None;
        }
        let left = &args[1];
        if left.is_empty() {
            return None;
        }
        let right = &args[2];
        Some(Box::new(Self {
            pattern: Regex::new(left).unwrap(),
            replacement: right.to_string(),
        }))
    }
}

impl Calculation for Derivation {
    fn apply(&self, spelling: Option<&mut Spelling>) -> bool {
        let Some(spelling) = spelling else {
            return false;
        };
        transformation::apply(spelling, &self.pattern, self.replacement.as_str())
    }

    fn deletion(&self) -> bool {
        false
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn type_name(&self) -> String {
        String::from("derive")
    }
}
