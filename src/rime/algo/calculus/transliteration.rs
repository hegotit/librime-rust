use std::any::Any;
use std::char::from_u32;
use std::collections::HashMap;

use crate::rime::algo::calculus::calculation::Calculation;
use crate::rime::algo::spelling::Spelling;

pub(crate) struct Transliteration {
    char_map: HashMap<u32, u32>,
}

impl Transliteration {
    pub(crate) fn parse(args: Vec<String>) -> Option<Box<dyn Calculation>> {
        if args.len() < 3 {
            return None;
        }
        let left = &args[1];
        let right = &args[2];
        let mut left_chars = left.chars();
        let mut right_chars = right.chars();
        let mut char_map = HashMap::new();
        while let (Some(left_char), Some(right_char)) = (left_chars.next(), right_chars.next()) {
            char_map.insert(left_char as u32, right_char as u32);
        }
        if left_chars.next().is_none() && right_chars.next().is_none() {
            Some(Box::new(Self { char_map }))
        } else {
            None
        }
    }
}

impl Calculation for Transliteration {
    fn apply(&self, spelling: Option<&mut Spelling>) -> bool {
        let Some(spelling) = spelling else {
            return false;
        };

        if spelling.str.is_empty() {
            return false;
        }

        let mut modified = false;
        let mut new_str = String::new();
        for c in spelling.str.chars() {
            if let Some(&new_c) = self.char_map.get(&(c as u32)) {
                new_str.push(from_u32(new_c).unwrap());
                modified = true;
            } else {
                new_str.push(c);
            }
        }
        if modified {
            spelling.str = new_str;
        }
        modified
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn type_name(&self) -> String {
        String::from("xlit")
    }
}
