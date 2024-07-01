use std::collections::BTreeMap;
use std::fs::File;
use std::io::Write;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;

use log::{error, info};

use crate::rime::algo::calculus::calculation::{Calculation, Calculus};
use crate::rime::algo::spelling::{Spelling, SpellingProperties};
use crate::rime::common::PathExt;
use crate::rime::config::config_types::ConfigList;

#[derive(Debug)]
pub struct Script(pub(crate) BTreeMap<String, Vec<Spelling>>);

impl Deref for Script {
    type Target = BTreeMap<String, Vec<Spelling>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Script {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Script {
    pub fn new() -> Self {
        Self { 0: BTreeMap::new() }
    }

    pub fn add_syllable(&mut self, syllable: &str) -> bool {
        if self.contains_key(syllable) {
            return false;
        }

        let spelling = Spelling::new(syllable);
        if let Some(spellings) = self.get_mut(syllable) {
            spellings.push(spelling);
        } else {
            self.insert(syllable.to_string(), vec![spelling]);
        }
        true
    }

    pub(crate) fn merge(
        &mut self,
        key: &str,
        input_props: &SpellingProperties,
        input_spellings: &[Spelling],
    ) {
        let spellings = self.entry(key.to_string()).or_insert_with(Vec::new);
        for new_spelling in input_spellings {
            let mut updated_spelling = new_spelling.clone();
            let updated_props = &mut updated_spelling.properties;

            if input_props.type_ > updated_props.type_ {
                updated_props.type_ = input_props.type_.clone();
            }
            updated_props.credibility += input_props.credibility;

            if !input_props.tips.is_empty() {
                updated_props.tips = input_props.tips.clone();
            }

            if let Some(existing_spelling) = spellings.iter_mut().find(|e| **e == *new_spelling) {
                let existing_props = &mut existing_spelling.properties;

                if updated_props.type_ < existing_props.type_ {
                    existing_props.type_ = updated_props.type_.clone();
                }
                if updated_props.credibility > existing_props.credibility {
                    existing_props.credibility = updated_props.credibility;
                }
                existing_props.tips.clear();
            } else {
                spellings.push(updated_spelling);
            }
        }
    }

    pub(crate) fn dump(&self, file_path: &PathExt) -> std::io::Result<()> {
        let mut file = File::create(file_path)?;
        for (key, values) in self.deref() {
            let mut first = true;
            for s in values {
                writeln!(
                    file,
                    "{}\t{}\t{}\t{}\t{}",
                    if first { key } else { "" },
                    s.str,
                    "-ac?!".chars().nth(s.properties.type_ as usize).unwrap(),
                    s.properties.credibility,
                    s.properties.tips
                )?;
                first = false;
            }
        }
        Ok(())
    }
}

pub struct Projection {
    calculation: Vec<Arc<dyn Calculation>>,
}

impl Projection {
    pub fn new() -> Self {
        Projection {
            calculation: Vec::new(),
        }
    }

    pub fn load(&mut self, settings: Option<Arc<ConfigList>>) -> bool {
        let Some(settings) = settings else {
            return false;
        };
        self.calculation.clear();
        let calc = Calculus::new();
        let mut success = true;
        for (i, _) in settings.seq.iter().enumerate() {
            if let Some(formula) = settings.get_str_at(i) {
                match calc.parse(formula) {
                    Some(x) => self.calculation.push(Arc::from(x)),
                    None => {
                        error!(
                            "Error loading spelling algebra definition #{}: '{}'.",
                            i + 1,
                            formula
                        );
                        success = false;
                        break;
                    }
                }
            } else {
                error!("Error loading formula #{}", i + 1);
                success = false;
                break;
            }
        }
        if !success {
            self.calculation.clear();
        }
        success
    }

    pub fn apply(&self, value: Option<&mut String>) -> bool {
        let Some(value) = value else {
            return false;
        };

        if value.is_empty() {
            return false;
        }

        let mut spelling = Spelling {
            str: value.clone(),
            ..Default::default()
        };

        let modified = self.calculation.iter().fold(false, |current_result, calc| {
            current_result | calc.apply(Some(&mut spelling))
        });

        if modified {
            *value = String::from(spelling.str);
        }
        modified
    }

    pub fn apply_script(&self, script: Option<&mut Script>) -> bool {
        let Some(script) = script else {
            return false;
        };

        if script.is_empty() {
            return false;
        }

        let mut modified = false;
        let default_props = SpellingProperties::default();

        for (round, calc) in self.calculation.iter().enumerate() {
            info!("Round #{}", round + 1);
            let mut new_script = Script::new();

            for (key, input_spellings) in &script.0 {
                let mut temp_spelling = Spelling {
                    str: key.clone(),
                    ..Default::default()
                };

                let applied = calc.apply(Some(&mut temp_spelling));
                modified |= applied;

                if applied {
                    if !calc.deletion() {
                        new_script.merge(key, &default_props, input_spellings);
                    }
                    if calc.addition() && !temp_spelling.str.is_empty() {
                        new_script.merge(
                            &temp_spelling.str,
                            &temp_spelling.properties,
                            input_spellings,
                        );
                    }
                } else {
                    new_script.merge(key, &default_props, input_spellings);
                }
            }
            *script = new_script;
        }
        modified
    }
}
