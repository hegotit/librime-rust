use crate::rime::algo::calculus::calculation::{Calculation, Calculus};
use crate::rime::algo::spelling::{Spelling, SpellingProperties};
use crate::rime::config::config_types::ConfigList;
use log::{error, info};
use std::collections::HashMap;
use std::fs::File;
use std::io::{Result, Write};
use std::path::Path;
use std::sync::Arc;

struct Script {
    map: HashMap<String, Vec<Spelling>>,
}

impl Script {
    pub(crate) fn new() -> Self {
        Script {
            map: HashMap::new(),
        }
    }

    pub(crate) fn add_syllable(&mut self, syllable: &str) -> bool {
        if self.map.contains_key(syllable) {
            return false;
        }

        let spelling = Spelling::new(syllable);
        if let Some(spellings) = self.map.get_mut(syllable) {
            spellings.push(spelling);
        } else {
            self.map.insert(syllable.to_string(), vec![spelling]);
        }
        true
    }

    pub(crate) fn merge(&mut self, s: &str, sp: &SpellingProperties, v: Vec<Spelling>) {
        let entry = self.map.entry(s.to_string()).or_insert(Vec::new());
        for x in v {
            let mut y = x.clone();
            let yy = &mut y.properties;
            if sp.type_ > yy.type_ {
                yy.type_ = sp.type_.clone();
            }
            yy.credibility += sp.credibility;
            if !sp.tips.is_empty() {
                yy.tips = sp.tips.clone();
            }

            if let Some(e) = entry.iter_mut().find(|e| **e == x) {
                let zz = &mut e.properties;
                if yy.type_ < zz.type_ {
                    zz.type_ = yy.type_.clone();
                }
                if yy.credibility > zz.credibility {
                    zz.credibility = yy.credibility;
                }
                zz.tips.clear();
            } else {
                entry.push(y);
            }
        }
    }

    pub(crate) fn dump(&self, file_path: &Path) -> Result<()> {
        let mut file = File::create(file_path)?;
        for (key, values) in &self.map {
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

struct Projection {
    calculation: Vec<Arc<dyn Calculation>>,
}

impl Projection {
    pub(crate) fn new() -> Self {
        Projection {
            calculation: Vec::new(),
        }
    }

    pub(crate) fn load(&mut self, settings: Option<Arc<ConfigList>>) -> bool {
        let Some(settings) = settings else {
            return false;
        };
        self.calculation.clear();
        let calc = Calculus::new();
        let mut success = true;
        for (i, _) in settings.seq.iter().enumerate() {
            let Some(value) = settings.get_value_at(i) else {
                error!("Error loading formula #{}", i + 1);
                success = false;
                break;
            };
            let formula = value.str();
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
        }
        if !success {
            self.calculation.clear();
        }
        success
    }

    pub(crate) fn apply(&self, value: Option<&mut String>) -> bool {
        let Some(value) = value else {
            return false;
        };

        if value.is_empty() {
            return false;
        }

        let mut modified = false;
        let mut spelling = Spelling {
            str: value.clone(),
            ..Default::default()
        };
        for calc in &self.calculation {
            calc.apply(Some(&mut spelling));
            modified = true;
        }
        if modified {
            *value = String::from(spelling.str);
        }
        modified
    }

    pub(crate) fn apply_script(&self, value: Option<&mut Script>) -> bool {
        let Some(value) = value else {
            return false;
        };

        if value.map.is_empty() {
            return false;
        }

        let mut modified = false;
        let mut round = 0;
        for calc in &self.calculation {
            round += 1;
            info!("Round #{}", round);
            let mut temp = Script::new();
            for (key, vec_spelling) in &value.map {
                let mut s = Spelling {
                    str: key.clone(),
                    ..Default::default()
                };
                let applied = calc.apply(Some(&mut s));
                if applied {
                    modified = true;
                    if !calc.deletion() {
                        temp.merge(key, &SpellingProperties::default(), vec_spelling.clone());
                    }
                    if calc.addition() && !s.str.is_empty() {
                        temp.merge(&s.str, &s.properties, vec_spelling.clone());
                    }
                } else {
                    temp.merge(key, &SpellingProperties::default(), vec_spelling.clone());
                }
            }
            *value = temp;
        }
        modified
    }
}
