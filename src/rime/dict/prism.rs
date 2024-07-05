use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::ops::Deref;
use std::sync::LazyLock;

use darts::searcher::SearchStep;
use darts::{DoubleArrayTrie, DoubleArrayTrieBuilder};
use log::{error, info};

use crate::rime::algo::algebra::Script;
use crate::rime::algo::spelling::{SpellingProperties, SpellingType};
use crate::rime::algo::SyllableId;
use crate::rime::common::PathExt;
use crate::rime::dict::mapped_file::MappedFile;

static DEFAULT_ALPHABET: LazyLock<[char; 26]> = LazyLock::new(|| {
    "abcdefghijklmnopqrstuvwxyz"
        .chars()
        .collect::<Vec<char>>()
        .try_into()
        .unwrap()
});

type Credibility = f32;

type SpellingMapItem = Vec<SpellingDescriptor>;
type SpellingMap = Vec<SpellingMapItem>;

#[derive(Default, Debug)]
pub(crate) struct SpellingDescriptor {
    pub(crate) syllable_id: SyllableId,
    pub(crate) type_: i32,
    credibility: Credibility,
    tips: String,
}

impl SpellingDescriptor {
    pub(crate) fn properties(&self) -> SpellingProperties {
        SpellingProperties::new(
            SpellingType::try_from(self.type_).unwrap_or_default(),
            0,
            self.credibility.into(),
            self.tips.to_string(),
        )
    }

    pub(crate) fn new(syllable_id: SyllableId) -> Self {
        Self {
            syllable_id,
            ..Default::default()
        }
    }
}

struct Metadata {
    format: String,
    dict_file_checksum: u32,
    schema_file_checksum: u32,
    num_syllables: u32,
    num_spellings: u32,
    alphabet: [char; 256],
}

pub struct Prism {
    mapped_file: MappedFile,
    trie: Option<DoubleArrayTrie>,
    metadata: Option<Metadata>,
    spelling_map: Option<SpellingMap>,
    format: f64,
}

impl Deref for Prism {
    type Target = MappedFile;

    fn deref(&self) -> &Self::Target {
        &self.mapped_file
    }
}

impl Prism {
    pub(crate) fn trie(&self) -> &Option<DoubleArrayTrie> {
        &self.trie
    }

    pub fn new(file_path: PathExt) -> Self {
        Self {
            mapped_file: MappedFile::new(file_path).unwrap(),
            trie: None,
            metadata: None,
            spelling_map: None,
            format: 0.0,
        }
    }

    // Given a key, search all the keys in the tree that share a common prefix with that key.
    pub fn common_prefix_search(&self, key: &str) -> Option<Vec<Match>> {
        if key.is_empty() {
            return None;
        }

        self.trie
            .as_ref()?
            .common_prefix_search(key)
            .map(|list| list.into_iter().map(Match::from).collect())
    }

    pub(crate) fn query_spelling(&self, spelling_id: usize) -> Option<&Vec<SpellingDescriptor>> {
        self.spelling_map.as_ref()?.get(spelling_id)
    }

    pub fn expand_search(&self, key: &str, limit: usize) -> Option<Vec<Match>> {
        let trie = self.trie().as_ref()?;
        let mut result: Vec<Match> = Vec::new();
        let mut count = 0;

        if let Some(value) = trie.exact_match_search(&key) {
            result.push((key.len(), value).into());

            count += 1;
            if limit > 0 && count >= limit {
                return Some(result);
            }
        } else {
            let mut searcher = trie.search(key);
            if let Some(SearchStep::Reject(_, _)) | None = searcher.next() {
                return None;
            }
        }

        let mut queue = VecDeque::new();
        queue.push_back(String::from(key));

        let alphabet: &[_] = match &self.metadata {
            Some(metadata) if self.format > 1.0 - f64::EPSILON => &metadata.alphabet,
            _ => DEFAULT_ALPHABET.deref(),
        };

        while let Some(query_key) = queue.pop_front() {
            let mut new_key = query_key.clone();
            for &ch in alphabet {
                new_key.push(ch);

                if let Some(value) = trie.exact_match_search(&new_key) {
                    result.push((new_key.len(), value).into());

                    queue.push_back(new_key.clone());

                    count += 1;
                    if limit > 0 && count >= limit {
                        return Some(result);
                    }
                } else {
                    let mut searcher = trie.search(&new_key);
                    if let Some(SearchStep::Incomplete(_, _)) = searcher.next() {
                        queue.push_back(new_key.clone());
                    }
                }

                new_key.pop();
            }
        }

        if result.is_empty() {
            None
        } else {
            Some(result)
        }
    }

    pub fn save(&self) -> bool {
        info!("saving prism file: {}", self.file_path());

        if self.trie.is_none() {
            error!("the trie has not been constructed!");
            return false;
        }

        //self.shrink_to_fit()

        todo!()
    }

    pub fn has_key(&self, key: &str) -> bool {
        self.get_value(key).is_some()
    }

    pub fn get_value(&self, key: &str) -> Option<usize> {
        self.trie.as_ref()?.exact_match_search(key)
    }

    pub fn build(&mut self, syllabary: &BTreeSet<&str>) -> bool {
        self.build_with_params(syllabary, None, 0, 0)
    }

    fn build_with_params(
        &mut self,
        syllabary: &BTreeSet<&str>,
        script: Option<&Script>,
        dict_file_checksum: u32,
        schema_file_checksum: u32,
    ) -> bool {
        // building double-array trie
        let num_syllables = syllabary.len();
        let num_spellings = if let Some(s) = script {
            s.len()
        } else {
            syllabary.len()
        };

        let mut keys = Vec::with_capacity(num_spellings);

        if let Some(script_map) = script {
            for (key, _) in script_map.iter() {
                keys.push(key.as_str());
            }
        } else {
            for syllable in syllabary.iter() {
                keys.push(syllable);
            }
        }

        self.trie = Some(DoubleArrayTrieBuilder::new().build(&keys));

        // alphabet
        let mut alphabet_set = BTreeSet::new();
        for key in keys.iter() {
            for c in key.chars() {
                alphabet_set.insert(c);
            }
        }

        let mut alphabet = ['\0'; 256];
        alphabet_set
            .iter()
            .take(alphabet.len())
            .enumerate()
            .for_each(|(i, &c)| alphabet[i] = c);

        // creating prism file
        let metadata = Metadata {
            dict_file_checksum,
            schema_file_checksum,
            num_syllables: num_syllables as u32,
            num_spellings: num_spellings as u32,
            alphabet,
            format: "Rime::Prism/3.0".to_string(),
        };
        self.metadata = Some(metadata);

        // building spelling map
        if let Some(script_map) = script {
            let mut syllable_to_id = BTreeMap::new();
            for (id, &syllable) in syllabary.iter().enumerate() {
                syllable_to_id.insert(syllable, id as SyllableId);
            }

            let mut spelling_map = Vec::with_capacity(num_spellings);

            for (_, descriptors) in script_map.deref() {
                let mut spelling_list = Vec::with_capacity(descriptors.len());
                for desc in descriptors {
                    if let Some(&syll_id) = syllable_to_id.get::<str>(&desc.str) {
                        spelling_list.push(SpellingDescriptor {
                            syllable_id: syll_id,
                            type_: desc.properties.type_ as i32,
                            credibility: desc.properties.credibility as f32,
                            tips: desc.properties.tips.clone(),
                        });
                    }
                }
                spelling_map.push(spelling_list);
            }

            self.spelling_map = Some(spelling_map);
        }

        true
    }
}

#[derive(Debug)]
pub struct Match {
    pub(crate) value: SyllableId,
    pub(crate) offset: usize,
}

impl From<(usize, usize)> for Match {
    fn from(t: (usize, usize)) -> Self {
        Self {
            offset: t.0,
            value: t.1 as SyllableId,
        }
    }
}

impl From<(usize, SyllableId)> for Match {
    fn from(t: (usize, SyllableId)) -> Self {
        Self {
            offset: t.0,
            value: t.1,
        }
    }
}

impl Match {
    pub fn value(&self) -> SyllableId {
        self.value
    }

    pub fn offset(&self) -> usize {
        self.offset
    }
}
