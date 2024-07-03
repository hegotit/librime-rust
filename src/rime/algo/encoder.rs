use std::ops::{Deref, DerefMut};
use std::sync::Arc;

use log::{error, info, warn};
use regex::Regex;

use crate::rime::config::config_component::Config;
use crate::rime::config::config_types::{ConfigList, ConfigMap, ConfigValue};

const ENCODER_DFS_LIMIT: i32 = 32;
const MAX_PHRASE_LENGTH: i32 = 32;

#[derive(Default)]
pub(crate) struct RawCode(Vec<String>);

impl Deref for RawCode {
    type Target = Vec<String>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for RawCode {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl RawCode {
    pub(crate) fn to_string(&self) -> String {
        self.join(" ")
    }

    pub(crate) fn set_from_string(&mut self, code_str: &str) {
        self.0 = code_str.split_whitespace().map(String::from).collect();
    }
}

pub(crate) trait PhraseCollector {
    fn create_entry(&self, phrase: &str, code_str: &str, value: &str);
    fn translate_word(&self, word: &str) -> Option<Vec<String>>;
}

struct DummyCollector;

impl PhraseCollector for DummyCollector {
    fn create_entry(&self, _phrase: &str, _code_str: &str, _value: &str) {
        todo!()
    }

    fn translate_word(&self, _word: &str) -> Option<Vec<String>> {
        None
    }
}

pub(crate) struct Encoder {
    collector: Arc<dyn PhraseCollector + Sync + Send>,
}

impl Encoder {
    pub(crate) fn new(collector: Arc<dyn PhraseCollector + Sync + Send>) -> Self {
        Self { collector }
    }

    pub(crate) fn load_settings(&self, _config: &Config) -> bool {
        false
    }

    pub(crate) fn encode_phrase(&self, _phrase: &str, _value: &str) -> bool {
        false
    }

    pub(crate) fn set_collector(&mut self, collector: Arc<dyn PhraseCollector + Sync + Send>) {
        self.collector = collector;
    }
}

// Aa : code at index 0 for character at index 0
// Az : code at index -1 for character at index 0
// Za : code at index 0 for character at index -1
#[derive(Clone)]
pub(crate) struct CodeCoords {
    char_index: i32,
    code_index: i32,
}

pub(crate) struct TableEncodingRule {
    min_word_length: i32,
    max_word_length: i32,
    coords: Vec<CodeCoords>,
}

pub(crate) struct TableEncoder {
    encoder: Encoder,
    loaded: bool,
    encoding_rules: Vec<TableEncodingRule>,
    exclude_patterns: Vec<Regex>,
    tail_anchor: String,
    max_phrase_length: i32,
}

impl Deref for TableEncoder {
    type Target = Encoder;

    fn deref(&self) -> &Self::Target {
        &self.encoder
    }
}

impl DerefMut for TableEncoder {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.encoder
    }
}

impl TableEncoder {
    pub(crate) fn new(collector: Option<Arc<dyn PhraseCollector + Sync + Send>>) -> Self {
        let encoder = Encoder::new(collector.unwrap_or_else(|| Arc::new(DummyCollector {})));
        Self {
            encoder,
            loaded: false,
            encoding_rules: Vec::new(),
            exclude_patterns: Vec::new(),
            tail_anchor: String::new(),
            max_phrase_length: 0,
        }
    }

    pub(crate) fn load_settings(&mut self, config: Option<&Config>) -> bool {
        self.loaded = false;
        self.max_phrase_length = 0;
        self.encoding_rules.clear();
        self.exclude_patterns.clear();
        self.tail_anchor.clear();

        let Some(config) = config else {
            return false;
        };

        self.process_rules(config);

        config.data.read().ok().and_then(|data| {
            let item = data
                .root
                .as_any()
                .downcast_ref::<ConfigMap>()?
                .get("encoder/exclude_patterns")?;

            let seq = &item.as_any().downcast_ref::<ConfigList>()?.seq;

            self.exclude_patterns
                .extend(seq.iter().flatten().filter_map(|pattern| {
                    pattern
                        .as_any()
                        .downcast_ref::<ConfigValue>()
                        .and_then(|pattern| Regex::new(pattern.str()).ok())
                }));

            Some(())
        });

        let string = config.get_string("encoder/tail_anchor");
        if !string.is_empty() {
            self.tail_anchor = string;
        }

        self.loaded = !self.encoding_rules.is_empty();
        self.loaded
    }

    fn process_rules(&mut self, config: &Config) {
        config.data.read().ok().and_then(|data| {
            let item = data
                .root
                .as_any()
                .downcast_ref::<ConfigMap>()?
                .get("encoder/rules")?;

            let seq = &item.as_any().downcast_ref::<ConfigList>()?.seq;

            for rule in seq.iter().flatten() {
                if let Some(rule) = rule.as_any().downcast_ref::<ConfigMap>() {
                    self.process_rule(rule);
                }
            }

            self.max_phrase_length = self.max_phrase_length.min(MAX_PHRASE_LENGTH);

            Some(())
        });
    }

    fn process_rule(&mut self, rule: &ConfigMap) {
        if let Some(formula) = rule.get_str("formula") {
            if !formula.is_empty() {
                let mut table_rule = TableEncodingRule {
                    min_word_length: 0,
                    max_word_length: 0,
                    coords: Vec::new(),
                };

                if self.parse_formula(formula, &mut table_rule) {
                    if self.set_lengths(rule, &mut table_rule) {
                        self.encoding_rules.push(table_rule);
                    }
                }
            }
        }
    }

    fn set_lengths(&mut self, rule: &ConfigMap, table_rule: &mut TableEncodingRule) -> bool {
        if let Some(length) = rule.get_int("length_equal") {
            table_rule.max_word_length = length;
            table_rule.min_word_length = length;
            self.update_max_phrase_length(length);
        } else {
            if let Some(range) = rule.get("length_in_range") {
                if let Some(range) = range
                    .as_any()
                    .downcast_ref::<ConfigList>()
                    .filter(|r| r.size() == 2)
                {
                    if let (Some(min), Some(max)) = (
                        range.get_value_at(0).and_then(|v| v.parse_int()),
                        range.get_value_at(1).and_then(|v| v.parse_int()),
                    ) {
                        if min <= max {
                            table_rule.min_word_length = min;
                            table_rule.max_word_length = max;
                            self.update_max_phrase_length(max);
                            return true;
                        }
                    }
                }
                error!("Invalid range");
                return false;
            }
        }

        true
    }

    fn update_max_phrase_length(&mut self, length: i32) {
        if self.max_phrase_length < length {
            self.max_phrase_length = length;
        }
    }

    fn parse_formula(&self, formula: &str, rule: &mut TableEncodingRule) -> bool {
        if formula.len() % 2 != 0 {
            error!("Bad formula: '{}'", formula);
            return false;
        }

        for chunk in formula.as_bytes().chunks(2) {
            if !chunk[0].is_ascii_uppercase() {
                error!("Invalid character index in formula: '{}'", formula);
                return false;
            }

            if !chunk[1].is_ascii_lowercase() {
                error!("Invalid code index in formula: '{}'", formula);
                return false;
            }

            let char_index = match chunk[0] as char {
                'A'..='T' => (chunk[0] - b'A') as i32,
                'U'..='Z' => (chunk[0] - b'Z' - 1) as i32,
                _ => return false,
            };

            let code_index = match chunk[1] as char {
                'a'..='t' => (chunk[1] - b'a') as i32,
                'u'..='z' => (chunk[1] - b'z' - 1) as i32,
                _ => return false,
            };

            rule.coords.push(CodeCoords {
                char_index,
                code_index,
            });
        }

        true
    }

    fn encode(&self, code: &RawCode) -> Option<String> {
        let num_syllables = code.0.len() as i32;
        for rule in &self.encoding_rules {
            if num_syllables < rule.min_word_length || num_syllables > rule.max_word_length {
                continue;
            }

            let mut result = Vec::new();
            let mut previous = CodeCoords {
                char_index: 0,
                code_index: 0,
            };
            let mut encoded = CodeCoords {
                char_index: 0,
                code_index: 0,
            };

            for current in &rule.coords {
                let mut c = current.clone();

                if c.char_index < 0 {
                    c.char_index += num_syllables;
                }

                if c.char_index >= num_syllables {
                    continue; // 'abc def' ~ 'Ca'
                }

                if c.char_index < 0 {
                    continue; // 'abc def' ~ 'Xa'
                }

                if current.char_index < 0 && c.char_index < encoded.char_index {
                    continue; // 'abc def' ~ '(AaBa)Ya'
                              // 'abc def' ~ '(AaBa)Aa' is OK
                }

                let start_index = if c.char_index == encoded.char_index {
                    encoded.code_index + 1
                } else {
                    0
                };

                c.code_index = self.calculate_code_index(
                    &code.0[c.char_index as usize],
                    c.code_index,
                    start_index,
                );

                if c.code_index >= code.0[c.char_index as usize].len() as i32 {
                    continue; // 'abc def' ~ 'Ad'
                }

                if c.code_index < 0 {
                    continue; // 'abc def' ~ 'Ax'
                }

                if (current.char_index < 0 || current.code_index < 0)
                    && c.char_index == encoded.char_index
                    && c.code_index <= encoded.code_index
                    && (current.char_index != previous.char_index
                        || current.code_index != previous.code_index)
                {
                    continue; // 'abc def' ~ '(AaBb)By', '(AaBb)Zb', '(AaZb)Zy'
                              // 'abc def' ~ '(AaZb)Zb' is OK
                              // 'abc def' ~ '(AaZb)Zz' is OK
                }

                if let Some(char_vec) = code.0.get(c.char_index as usize) {
                    let bytes = char_vec.as_bytes();
                    if let Some(byte) = bytes.get(c.code_index as usize) {
                        result.push(*byte);
                    }
                }

                if let Some(byte) = code
                    .get(c.char_index as usize)
                    .and_then(|f| f.as_bytes().get(c.code_index as usize))
                {
                    result.push(*byte);
                }

                previous = current.clone();
                encoded = c.clone();
            }

            if !result.is_empty() {
                if let Ok(s) = String::from_utf8(result) {
                    return Some(s);
                }
            }
        }

        None
    }

    fn encode_phrase(&self, phrase: &str, value: &str) -> bool {
        let phrase_length = phrase.chars().count();

        if phrase_length as i32 > self.max_phrase_length {
            return false;
        }

        let mut code = RawCode::default();
        let mut limit = ENCODER_DFS_LIMIT;

        self.dfs_encode(phrase, value, 0, &mut code, &mut limit)
    }

    fn dfs_encode(
        &self,
        phrase: &str,
        value: &str,
        start_pos: usize,
        code: &mut RawCode,
        limit: &mut i32,
    ) -> bool {
        if start_pos == phrase.len() {
            *limit -= 1;

            if let Some(encoded) = self.encode(code) {
                info!(
                    "Encode '{}': [{}] -> [{}]",
                    phrase,
                    code.to_string(),
                    encoded
                );
                self.collector.create_entry(phrase, &encoded, value);
                return true;
            } else {
                warn!("Failed to encode '{}': [{}]", phrase, code.to_string());
                return false;
            }
        }

        let slice = &phrase[start_pos..];
        let char_len = slice.chars().next().map(|c| c.len_utf8()).unwrap_or(0);
        let end_pos = start_pos + char_len;
        let word = &phrase[start_pos..end_pos];

        let translations = self.collector.translate_word(word);
        if let Some(translations) = translations {
            for translation in translations {
                if self.is_code_excluded(&translation) {
                    continue;
                }

                code.push(translation);
                let ok = self.dfs_encode(phrase, value, start_pos + word.len(), code, limit);
                code.pop();

                if *limit <= 0 {
                    return ok;
                }
            }
        }

        false
    }

    fn is_code_excluded(&self, code: &str) -> bool {
        self.exclude_patterns
            .iter()
            .any(|pattern| pattern.is_match(code))
    }

    fn calculate_code_index(&self, code: &str, index: i32, start: i32) -> i32 {
        let mut index = index;
        info!("code = {}, index = {}, start = {}", code, index, start);
        let n = code.len() as i32;
        let mut k = 0;

        let tail_anchor_bytes = self.tail_anchor.as_bytes();
        let code_bytes = code.as_bytes();
        if index < 0 {
            // 'ab|cd|ef|g' ~ '(Aa)Az' -> 'ab'; start = 1, index = -1
            // 'ab|cd|ef|g' ~ '(AaAb)Az' -> 'abd'; start = 4, index = -1
            // 'ab|cd|ef|g' ~ '(AaAb)Ay' -> 'abc'; start = 4, index = -2
            k = n - 1;
            if let Some(tail) = code[(start + 1) as usize..].find(&self.tail_anchor) {
                k = tail as i32 + start;
            }

            while {
                index += 1;
                index
            } < 0
            {
                while {
                    k -= 1;
                    k
                } >= 0
                    && tail_anchor_bytes.contains(&code_bytes[k as usize])
                {}
            }
        } else {
            // 'ab|cd|ef|g' ~ '(AaAb)Ac' -> 'abc'; index = 2
            while index > 0 {
                index -= 1;
                while {
                    k += 1;
                    k
                } < n
                    && tail_anchor_bytes.contains(&code_bytes[k as usize])
                {}
            }
        }

        k
    }
}

pub(crate) struct ScriptEncoder {
    encoder: Encoder,
}

impl Deref for ScriptEncoder {
    type Target = Encoder;

    fn deref(&self) -> &Self::Target {
        &self.encoder
    }
}

impl DerefMut for ScriptEncoder {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.encoder
    }
}

impl ScriptEncoder {
    pub(crate) fn new(collector: Arc<dyn PhraseCollector + Sync + Send>) -> Self {
        let encoder = Encoder::new(collector);
        Self { encoder }
    }

    pub(crate) fn encode_phrase(&self, phrase: &str, value: &str) -> bool {
        let phrase_length = phrase.chars().count();

        if phrase_length as i32 > MAX_PHRASE_LENGTH {
            return false;
        }

        let mut code = RawCode::default();
        let mut limit = ENCODER_DFS_LIMIT;
        self.dfs_encode(phrase, value, 0, &mut code, &mut limit)
    }

    fn dfs_encode(
        &self,
        phrase: &str,
        value: &str,
        start_pos: usize,
        code: &mut RawCode,
        limit: &mut i32,
    ) -> bool {
        if start_pos == phrase.len() {
            *limit -= 1;
            self.collector
                .create_entry(phrase, &code.to_string(), value);
            return true;
        }

        for k in (1..=phrase.len() - start_pos).rev() {
            let word = &phrase[start_pos..start_pos + k];
            if let Some(translations) = self.collector.translate_word(word) {
                for translation in translations {
                    code.push(translation);
                    let ok = self.dfs_encode(phrase, value, start_pos + k, code, limit);
                    code.pop();
                    if *limit <= 0 {
                        return ok;
                    }
                }
            }
        }

        false
    }
}
