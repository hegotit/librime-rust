use std::collections::{HashMap, HashSet, VecDeque};
use std::ops::{Deref, DerefMut};
use std::sync::LazyLock;

use crate::rime::algo::SyllableId;
use crate::rime::dict::prism::Prism;
use common_macros::hash_set;

static KEYBOARD_MAP: LazyLock<HashMap<char, HashSet<char>>> = LazyLock::new(|| {
    [
        ('1', hash_set! {'2', 'q', 'w'}),
        ('2', hash_set! {'1', '3', 'q', 'w', 'e'}),
        ('3', hash_set! {'2', '4', 'w', 'e', 'r'}),
        ('4', hash_set! {'3', '5', 'e', 'r', 't'}),
        ('5', hash_set! {'4', '6', 'r', 't', 'y'}),
        ('6', hash_set! {'5', '7', 't', 'y', 'u'}),
        ('7', hash_set! {'6', '8', 'y', 'u', 'i'}),
        ('8', hash_set! {'7', '9', 'u', 'i', 'o'}),
        ('9', hash_set! {'8', '0', 'i', 'o', 'p'}),
        ('0', hash_set! {'9', '-', 'o', 'p', '['}),
        ('-', hash_set! {'0', '=', 'p', '[', ']'}),
        ('=', hash_set! {'-', '[', ']', '\\'}),
        ('q', hash_set! {'w'}),
        ('w', hash_set! {'q', 'e'}),
        ('e', hash_set! {'w', 'r'}),
        ('r', hash_set! {'e', 't'}),
        ('t', hash_set! {'r', 'y'}),
        ('y', hash_set! {'t', 'u'}),
        ('u', hash_set! {'y', 'i'}),
        ('i', hash_set! {'u', 'o'}),
        ('o', hash_set! {'i', 'p'}),
        ('p', hash_set! {'o', '['}),
        ('[', hash_set! {'p', ']'}),
        (']', hash_set! {'[', '\\'}),
        ('\\', hash_set! {']'}),
        ('a', hash_set! {'s'}),
        ('s', hash_set! {'a', 'd'}),
        ('d', hash_set! {'s', 'f'}),
        ('f', hash_set! {'d', 'g'}),
        ('g', hash_set! {'f', 'h'}),
        ('h', hash_set! {'g', 'j'}),
        ('j', hash_set! {'h', 'k'}),
        ('k', hash_set! {'j', 'l'}),
        ('l', hash_set! {'k', ';'}),
        (';', hash_set! {'l', '\''}),
        ('\'', hash_set! {';'}),
        ('z', hash_set! {'x'}),
        ('x', hash_set! {'z', 'c'}),
        ('c', hash_set! {'x', 'v'}),
        ('v', hash_set! {'c', 'b'}),
        ('b', hash_set! {'v', 'n'}),
        ('n', hash_set! {'b', 'm'}),
        ('m', hash_set! {'n', ','}),
        (',', hash_set! {'m', '.'}),
        ('.', hash_set! {',', '/'}),
        ('/', hash_set! {'.'}),
    ]
    .iter()
    .cloned()
    .collect()
});

pub struct Correction {
    distance: usize,
    pub(crate) offset: usize,
}

pub struct Corrections(pub(crate) HashMap<SyllableId, Correction>);

impl Corrections {
    pub(crate) fn new() -> Self {
        Self(HashMap::new())
    }
}

impl Deref for Corrections {
    type Target = HashMap<SyllableId, Correction>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Corrections {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Corrections {
    /// Update for better correction
    #[inline]
    fn alter(&mut self, syllable: SyllableId, correction: Correction) {
        if !self.contains_key(&syllable) || correction.distance < self[&syllable].distance {
            self.insert(syllable, correction);
        }
    }
}

#[derive(Debug)]
struct Record {
    idx: usize,
    distance: usize,
    ch: char,
}

impl Record {
    fn new(idx: usize, distance: usize, ch: char) -> Self {
        Self { idx, distance, ch }
    }
}

pub trait Corrector {
    fn tolerance_search(
        &self,
        prism: &Prism,
        key: &str,
        results: &mut Corrections,
        tolerance: usize,
    );
}

#[derive(Default)]
pub struct NearSearchCorrector;

impl Corrector for NearSearchCorrector {
    fn tolerance_search(
        &self,
        prism: &Prism,
        key: &str,
        results: &mut Corrections,
        threshold: usize,
    ) {
        if key.is_empty() {
            return;
        }

        let Some(trie) = prism.trie() else {
            return;
        };

        let mut queue = VecDeque::new();
        for (index, ch) in key.char_indices() {
            queue.push_back(Record::new(index, index, ch));

            if index < threshold {
                if let Some(substitutions) = KEYBOARD_MAP.get(&ch) {
                    for &subst in substitutions {
                        queue.push_back(Record::new(index, index + 1, subst));
                    }
                }
            }
        }

        for rec in queue {
            let next_boundary = rec.idx + rec.ch.len_utf8();

            let actual_query_key =
                format!("{}{}{}", &key[..rec.idx], rec.ch, &key[next_boundary..]);

            if let Some(list) = trie.common_prefix_search(&actual_query_key) {
                for (offset, matched) in list {
                    results.alter(
                        matched as SyllableId,
                        Correction {
                            distance: rec.distance,
                            offset,
                        },
                    );
                }
            }
        }
    }
}
