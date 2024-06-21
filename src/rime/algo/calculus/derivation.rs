use std::{any::Any, sync::Arc};

use regex::Regex;

use crate::rime::algo::calculus::calculation::Calculation;

pub struct Derivation {
    pattern: Regex,
    replacement: String,
}

impl Derivation {
    pub fn parse(args: Vec<String>) -> Option<Arc<dyn Calculation>> {
        if args.len() < 3 {
            return None;
        }
        let left = &args[1];
        if left.is_empty() {
            return None;
        }
        let right = &args[2];
        Some(Arc::new(Self {
            pattern: Regex::new(left).unwrap(),
            replacement: right.to_string(),
        }))
    }
}

impl Calculation for Derivation {
    fn deletion(&self) -> bool {
        false
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
