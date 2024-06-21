use std::any::Any;
use std::collections::HashMap;

use crate::rime::algo::calculus::abbreviation::Abbreviation;
use crate::rime::algo::calculus::derivation::Derivation;
use crate::rime::algo::calculus::erasure::Erasure;
use crate::rime::algo::calculus::fuzzing::Fuzzing;
use crate::rime::algo::calculus::transformation::Transformation;
use crate::rime::algo::calculus::transliteration::Transliteration;
use crate::rime::algo::spelling::Spelling;

pub trait Calculation: Any {
    fn apply(&self, _spelling: Option<&mut Spelling>) -> bool {
        false
    }

    fn addition(&self) -> bool {
        true
    }

    fn deletion(&self) -> bool {
        true
    }

    fn as_any(&self) -> &dyn Any;

    fn type_name(&self) -> String;
}

pub struct Calculus {
    factories: HashMap<String, Box<dyn Fn(Vec<String>) -> Option<Box<dyn Calculation>>>>,
}

impl Calculus {
    pub fn new() -> Self {
        let mut calculus = Self {
            factories: HashMap::new(),
        };
        calculus.register("xlit", Box::new(Transliteration::parse));
        calculus.register("xform", Box::new(Transformation::parse));
        calculus.register("erase", Box::new(Erasure::parse));
        calculus.register("derive", Box::new(Derivation::parse));
        calculus.register("fuzz", Box::new(Fuzzing::parse));
        calculus.register("abbrev", Box::new(Abbreviation::parse));
        calculus
    }

    pub(crate) fn register(
        &mut self,
        token: &str,
        factory: Box<dyn Fn(Vec<String>) -> Option<Box<dyn Calculation>>>,
    ) {
        self.factories.insert(token.to_string(), factory);
    }

    pub fn parse(&self, definition: &str) -> Option<Box<dyn Calculation>> {
        if let Some(sep) = definition.chars().find(|c| !c.is_ascii_lowercase()) {
            let args: Vec<String> = definition.split(sep).map(|s| s.to_string()).collect();
            if let Some(factory) = self.factories.get(&args[0]) {
                return factory(args);
            }
        }
        None
    }
}
