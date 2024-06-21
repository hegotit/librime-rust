use std::any::Any;
use std::sync::Arc;

use regex::Regex;

use crate::rime::algo::calculus::calculation::Calculation;
use crate::rime::algo::spelling::Spelling;

pub struct Erasure {
    pattern: Regex,
}

impl Erasure {
    pub fn parse(args: Vec<String>) -> Option<Arc<dyn Calculation>> {
        if args.len() < 2 {
            return None;
        }
        let pattern = &args[1];
        if pattern.is_empty() {
            return None;
        }
        Some(Arc::new(Self {
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
}
