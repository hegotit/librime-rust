use std::any::Any;

use regex::Regex;

use crate::rime::algo::calculus::calculation::Calculation;
use crate::rime::algo::spelling::Spelling;

pub(crate) struct Transformation {
    pattern: Regex,
    replacement: String,
}

impl Transformation {
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

impl Calculation for Transformation {
    fn apply(&self, spelling: Option<&mut Spelling>) -> bool {
        let Some(spelling) = spelling else {
            return false;
        };
        apply(spelling, &self.pattern, self.replacement.as_str())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn type_name(&self) -> String {
        String::from("xform")
    }
}

pub(crate) fn apply(spelling: &mut Spelling, pattern: &Regex, replacement: &str) -> bool {
    if spelling.str.is_empty() {
        return false;
    }

    let result = pattern.replace_all(&spelling.str, replacement);
    if result == spelling.str {
        false
    } else {
        spelling.str = result.to_string();
        true
    }
}
