use std::cmp::Ordering;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::ops::{Deref, DerefMut};
use std::sync::{Arc, RwLock};

use crate::rime::algo::SyllableId;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Code(Vec<SyllableId>);

impl Deref for Code {
    type Target = Vec<SyllableId>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Code {
    const INDEX_CODE_MAX_LENGTH: usize = 3;

    fn new() -> Self {
        Self(Vec::new())
    }

    fn create_index(&self, index_code: Option<&mut Self>) {
        if let Some(index_code) = index_code {
            let index_code_size = self.len().min(Self::INDEX_CODE_MAX_LENGTH);
            index_code.0 = self[..index_code_size].to_vec();
        }
    }

    fn to_string(&self) -> String {
        self.iter()
            .map(|id| id.to_string())
            .collect::<Vec<String>>()
            .join(",")
    }
}

#[derive(Debug)]
struct ShortDictEntry {
    text: String,
    code: Code, // Multi-syllable code from prism
    weight: f64,
}

impl ShortDictEntry {
    fn new(text: String, code: Code, weight: f64) -> Self {
        Self { text, code, weight }
    }
}

impl PartialEq for ShortDictEntry {
    fn eq(&self, other: &Self) -> bool {
        self.weight == other.weight && self.text == other.text;
        todo!()
    }
}

impl Eq for ShortDictEntry {}

impl PartialOrd for ShortDictEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ShortDictEntry {
    // Sort different entries sharing the same code by weight desc.
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .weight
            .partial_cmp(&self.weight)
            .unwrap_or(Ordering::Equal);
        todo!()
    }
}

#[derive(Debug)]
struct DictEntry {
    text: String,
    comment: String,
    preedit: String,
    code: Code,          // multi-syllable code from prism
    custom_code: String, // user defined code
    weight: f64,
    commit_count: i32,
    remaining_code_length: i32,
    matching_code_size: i32,
}

impl DictEntry {
    fn to_short(&self) -> ShortDictEntry {
        ShortDictEntry::new(self.text.clone(), self.code.clone(), self.weight)
    }

    fn is_exact_match(&self) -> bool {
        self.matching_code_size == 0 || self.matching_code_size == self.code.len() as i32
    }

    fn is_predictive_match(&self) -> bool {
        self.matching_code_size != 0 && self.matching_code_size < self.code.len() as i32
    }
}

impl PartialEq for DictEntry {
    fn eq(&self, other: &Self) -> bool {
        self.weight == other.weight && self.text == other.text;
        todo!()
    }
}

impl Eq for DictEntry {}

impl PartialOrd for DictEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for DictEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .weight
            .partial_cmp(&self.weight)
            .unwrap_or(Ordering::Equal);
        todo!()
    }
}

#[derive(Default)]
struct ShortDictEntryList(Vec<Arc<ShortDictEntry>>);

impl Deref for ShortDictEntryList {
    type Target = Vec<Arc<ShortDictEntry>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ShortDictEntryList {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl ShortDictEntryList {
    fn sort_range(&mut self, start: usize, count: usize) {
        if start >= self.len() {
            return;
        }

        let end = (start + count).min(self.len());
        self[start..end].sort();
    }
}

struct DictEntryList(Vec<Arc<DictEntry>>);

impl Deref for DictEntryList {
    type Target = Vec<Arc<DictEntry>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for DictEntryList {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl DictEntryList {
    fn sort_range(&mut self, start: usize, count: usize) {
        if start >= self.len() {
            return;
        }

        let end = (start + count).min(self.0.len());
        self.0[start..end].sort();
    }
}

type DictEntryFilter = Box<dyn Fn(&Arc<DictEntry>) -> bool>;

struct DictEntryFilterBinder {
    filter: Option<DictEntryFilter>,
}

impl DictEntryFilterBinder {
    fn new() -> Self {
        Self { filter: None }
    }

    fn add_filter(&mut self, filter: DictEntryFilter) {
        if let Some(existing_filter) = self.filter.take() {
            self.filter = Some(Box::new(move |entry| {
                existing_filter(entry) && filter(entry)
            }));
        } else {
            self.filter = Some(filter);
        }
    }
}

struct VocabularyPage {
    entries: Arc<RwLock<ShortDictEntryList>>,
    next_level: Option<SharedVocabulary>,
}

impl VocabularyPage {
    fn new() -> Self {
        Self {
            entries: Arc::new(RwLock::new(ShortDictEntryList::default())),
            next_level: None,
        }
    }
}

struct SharedVocabulary(Arc<RwLock<Vocabulary>>);

impl Deref for SharedVocabulary {
    type Target = Arc<RwLock<Vocabulary>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for SharedVocabulary {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

struct Vocabulary(BTreeMap<i32, VocabularyPage>);

impl Deref for Vocabulary {
    type Target = BTreeMap<i32, VocabularyPage>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Vocabulary {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Vocabulary {
    fn new() -> Self {
        Self { 0: BTreeMap::new() }
    }

    fn locate_entries(self, code: &Code) -> bool {
        //let mut vocab = self;
        //for (i, syllable_id) in code.iter().enumerate() {
        //    let key = if i < Code::INDEX_CODE_MAX_LENGTH {
        //        *syllable_id
        //    } else {
        //        -1
        //    };

        //    let mut vocab_ref = vocab.write().unwrap();

        //    let page = vocab_ref
        //        .entry(key)
        //        .or_insert_with(|| VocabularyPage::new());

        //    if i == code.len() - 1 || i == Code::INDEX_CODE_MAX_LENGTH {
        //        Some(Arc::clone(&page.entries));
        //        return true;
        //    } else {
        //        vocab = page
        //            .next_level
        //            .get_or_insert_with(|| SharedVocabulary::new());
        //    }
        //}
        //false
        todo!()
    }

    fn sort_homophones(&mut self) {
        //for page in self.values_mut() {
        //    page.entries.sort();
        //    if let Some(ref mut next_level) = page.next_level.take() {
        //        Arc::make_mut(next_level).sort_homophones();
        //    }
        //}

        todo!()
    }
}

type ReverseLookupTable = HashMap<String, HashSet<String>>;

impl Display for Code {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.to_string())
    }
}
