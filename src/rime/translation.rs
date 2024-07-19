use crate::rime::candidate::{Candidate, CandidateList, CandidateQueue};
use log::{error, info, warn};
use std::collections::{HashSet, VecDeque};
use std::sync::{Arc, RwLock};

pub(crate) trait Translation {
    // A translation may contain multiple results, functioning
    // somewhat like a generator of candidates.
    fn next(&mut self) -> Option<Arc<dyn Candidate>>;

    fn peek(&self) -> Option<Arc<dyn Candidate>>;

    // Should it provide the next candidate (negative or zero) or
    // should it yield to other translations (positive)?
    fn compare(&self, other: Option<&Arc<RwLock<dyn Translation>>>) -> i32 {
        if let Some(other) = other {
            if let Ok(other) = other.read() {
                if other.exhausted() {
                    return -1;
                }
                if self.exhausted() {
                    return 1;
                }

                match (self.peek(), other.peek()) {
                    (Some(ours), Some(theirs)) => ours.as_ref().cmp(theirs.as_ref()) as i32,
                    _ => 1,
                }
            } else {
                return -1;
            }
        } else {
            return -1;
        }
    }

    fn exhausted(&self) -> bool;
}

struct UniqueTranslation {
    candidate: Option<Arc<dyn Candidate>>,
    exhausted: bool,
}

impl UniqueTranslation {
    fn new(candidate: Option<Arc<dyn Candidate>>) -> Self {
        let exhausted = candidate.is_none();
        Self {
            candidate,
            exhausted,
        }
    }

    fn set_exhausted(&mut self, exhausted: bool) {
        self.exhausted = exhausted;
    }
}

impl Translation for UniqueTranslation {
    fn next(&mut self) -> Option<Arc<dyn Candidate>> {
        if self.exhausted {
            None
        } else {
            self.set_exhausted(true);
            self.candidate.clone()
        }
    }

    fn peek(&self) -> Option<Arc<dyn Candidate>> {
        if self.exhausted {
            None
        } else {
            self.candidate.clone()
        }
    }

    fn exhausted(&self) -> bool {
        self.exhausted
    }
}

struct FifoTranslation {
    candies: CandidateList,
    cursor: usize,
    exhausted: bool,
}

impl FifoTranslation {
    fn new() -> Self {
        Self {
            candies: CandidateList::new(),
            cursor: 0,
            exhausted: true,
        }
    }

    fn append(&mut self, candy: Option<Arc<dyn Candidate>>) {
        self.candies.push(candy);
        self.set_exhausted(false);
    }

    fn size(&self) -> usize {
        self.candies.len() - self.cursor
    }

    fn set_exhausted(&mut self, exhausted: bool) {
        self.exhausted = exhausted;
    }
}

impl Translation for FifoTranslation {
    fn next(&mut self) -> Option<Arc<dyn Candidate>> {
        if self.exhausted {
            return None;
        }

        if self.cursor + 1 >= self.candies.len() {
            self.set_exhausted(true);
        }
        self.candies[self.cursor].clone()
    }

    fn peek(&self) -> Option<Arc<dyn Candidate>> {
        if self.exhausted {
            return None;
        }

        self.candies[self.cursor].clone()
    }

    fn exhausted(&self) -> bool {
        self.exhausted
    }
}

struct UnionTranslation {
    translations: VecDeque<Arc<RwLock<dyn Translation>>>,
    exhausted: bool,
}

impl UnionTranslation {
    fn new() -> Self {
        Self {
            translations: VecDeque::new(),
            exhausted: true,
        }
    }

    fn add_translation(&mut self, translation: Option<Arc<RwLock<dyn Translation>>>) {
        if let Some(arc) = translation {
            let cloned = arc.clone();
            if let Ok(translation) = arc.read() {
                if !translation.exhausted() {
                    self.translations.push_back(cloned);
                    self.exhausted = false;
                }
            } else {
                error!("Failed to acquire read lock");
            }
        }
    }

    fn set_exhausted(&mut self, exhausted: bool) {
        self.exhausted = exhausted;
    }
}

impl Translation for UnionTranslation {
    fn next(&mut self) -> Option<Arc<dyn Candidate>> {
        if self.exhausted {
            return None;
        }

        if let Some(front) = self.translations.pop_front() {
            let cloned = front.clone();
            match front.write() {
                Ok(mut translation) => {
                    let candidate = translation.next();
                    if !translation.exhausted() {
                        self.translations.push_front(cloned);
                    }
                    return candidate;
                }
                Err(_) => {
                    error!("Failed to acquire write lock");
                    self.translations.push_front(cloned);
                }
            }
        }

        if self.translations.is_empty() {
            self.set_exhausted(true);
        }

        None
    }

    fn peek(&self) -> Option<Arc<dyn Candidate>> {
        if self.exhausted {
            return None;
        }
        if let Some(front) = self.translations.front() {
            return match front.read() {
                Ok(translation) => translation.peek(),
                Err(_) => {
                    error!("Failed to acquire read lock");
                    None
                }
            };
        }
        None
    }

    fn exhausted(&self) -> bool {
        self.exhausted
    }
}

#[derive(Clone)]
pub(crate) struct MergedTranslation {
    previous_candidates: CandidateList,
    translations: Vec<Arc<RwLock<dyn Translation>>>,
    elected: usize,
    exhausted: bool,
}

impl MergedTranslation {
    pub(crate) fn new(previous_candidates: CandidateList) -> Self {
        Self {
            previous_candidates,
            translations: Vec::new(),
            elected: 0,
            exhausted: true,
        }
    }

    pub(crate) fn size(&self) -> usize {
        self.translations.len()
    }

    fn set_exhausted(&mut self, exhausted: bool) {
        self.exhausted = exhausted;
    }

    fn elect(&mut self) {
        if self.translations.is_empty() {
            self.set_exhausted(true);
            return;
        }

        let len = self.translations.len();
        let mut indexes_to_remove = HashSet::new();
        let mut current_index = 0;

        'outer: while current_index < len {
            while indexes_to_remove.contains(&current_index) {
                current_index += 1;
                if current_index >= len {
                    break 'outer;
                }
            }

            let current = &self.translations[current_index];
            let mut next_index = current_index + 1;
            while indexes_to_remove.contains(&next_index) {
                next_index += 1;
                if next_index > len {
                    break;
                }
            }

            let next = self.translations.get(next_index);

            if let Ok(current_readable) = current.read() {
                if current_readable.compare(next) <= 0 {
                    if current_readable.exhausted() {
                        indexes_to_remove.insert(current_index);
                        current_index = 0;
                        continue;
                    }
                    break;
                }
            }
            current_index += 1;
        }

        let mut sorted_indexes_to_remove: Vec<_> = indexes_to_remove.into_iter().collect();
        sorted_indexes_to_remove.sort_unstable();
        sorted_indexes_to_remove.reverse();

        let mut counts_below_index = 0;

        for index in sorted_indexes_to_remove {
            if index < index {
                counts_below_index += 1;
            }

            if index < len {
                self.translations.remove(index);
            }
        }

        self.elected = current_index - counts_below_index;

        if current_index >= self.translations.len() {
            warn!("Failed to elect a winner translation");
            self.exhausted = true;
        } else {
            self.exhausted = false;
        }
    }

    pub(crate) fn add_translation(&mut self, translation: Option<Arc<RwLock<dyn Translation>>>) {
        if let Some(arc) = translation {
            let cloned = arc.clone();
            if let Ok(translation) = arc.read() {
                if !translation.exhausted() {
                    self.translations.push(cloned);
                    self.elect();
                }
            } else {
                error!("Failed to acquire read lock");
            }
        }
    }
}

impl Translation for MergedTranslation {
    fn next(&mut self) -> Option<Arc<dyn Candidate>> {
        if self.exhausted {
            return None;
        }

        let elected = self.translations[self.elected].clone();
        let result = match elected.write() {
            Ok(mut translation) => {
                let candidate = translation.next();
                if translation.exhausted() {
                    info!("Translation #{} has been exhausted", self.elected);
                    self.translations.remove(self.elected);
                }
                candidate
            }
            Err(_) => {
                error!("Failed to acquire write lock");
                None
            }
        };

        self.elect();
        result
    }

    fn peek(&self) -> Option<Arc<dyn Candidate>> {
        if self.exhausted {
            return None;
        }

        let elected = &self.translations[self.elected];
        return match elected.read() {
            Ok(translation) => translation.peek(),
            Err(_) => {
                error!("Failed to acquire read lock");
                None
            }
        };
    }

    fn exhausted(&self) -> bool {
        self.exhausted
    }
}

struct CacheTranslation {
    translation: Option<Arc<RwLock<dyn Translation>>>,
    cache: Option<Arc<dyn Candidate>>,
    exhausted: bool,
}

impl CacheTranslation {
    fn new(translation: Option<Arc<RwLock<dyn Translation>>>) -> Self {
        let (exhausted, cache) = match translation.as_ref().and_then(|t| t.read().ok()) {
            Some(translation) => (translation.exhausted(), translation.peek()),
            None => (false, None),
        };

        Self {
            translation,
            cache,
            exhausted,
        }
    }

    fn set_exhausted(&mut self, exhausted: bool) {
        self.exhausted = exhausted;
    }

    fn peek_candidate_text(&mut self) -> Option<String> {
        if let Some(peeked) = self.peek() {
            Some(peeked.text().to_string())
        } else {
            None
        }
    }
}

impl Translation for CacheTranslation {
    fn next(&mut self) -> Option<Arc<dyn Candidate>> {
        if self.exhausted {
            return None;
        }

        let (exhausted, cache) = match self.translation.as_ref().and_then(|t| t.write().ok()) {
            Some(mut translation) => (translation.exhausted(), translation.next()),
            None => {
                error!("Failed to acquire write lock");
                (false, None)
            }
        };

        self.exhausted = exhausted;
        self.cache = cache;
        self.cache.clone()
    }

    fn peek(&self) -> Option<Arc<dyn Candidate>> {
        if self.exhausted {
            None
        } else {
            self.cache.clone()
        }
    }

    fn exhausted(&self) -> bool {
        self.exhausted
    }
}

struct DistinctTranslation {
    cache_translation: CacheTranslation,
    candidate_set: HashSet<String>,
}

impl DistinctTranslation {
    fn new(translation: Option<Arc<RwLock<dyn Translation>>>) -> Self {
        Self {
            cache_translation: CacheTranslation::new(translation),
            candidate_set: HashSet::new(),
        }
    }
}

impl Translation for DistinctTranslation {
    fn next(&mut self) -> Option<Arc<dyn Candidate>> {
        if self.cache_translation.exhausted() {
            return None;
        }

        let peeked = self.cache_translation.peek();
        if let Some(candidate) = &peeked {
            self.candidate_set.insert(candidate.text().to_string());
        }
        loop {
            self.cache_translation.next();
            if self.cache_translation.exhausted() {
                break;
            }

            match self.cache_translation.peek_candidate_text() {
                Some(candidate_text) if self.candidate_set.contains(&candidate_text) => continue,
                _ => break,
            }
        }
        peeked
    }

    fn peek(&self) -> Option<Arc<dyn Candidate>> {
        self.cache_translation.peek()
    }

    fn exhausted(&self) -> bool {
        self.cache_translation.exhausted()
    }
}

struct PrefetchTranslation {
    translation: Box<dyn Translation>,
    cache: CandidateQueue,
    exhausted: bool,
}

impl PrefetchTranslation {
    fn new(translation: Box<dyn Translation>) -> Self {
        let exhausted = translation.exhausted();
        Self {
            translation,
            cache: VecDeque::new(),
            exhausted,
        }
    }

    fn replenish(&self) -> bool {
        false
    }

    fn set_exhausted(&mut self, exhausted: bool) {
        self.exhausted = exhausted
    }
}

impl Translation for PrefetchTranslation {
    fn next(&mut self) -> Option<Arc<dyn Candidate>> {
        if self.exhausted {
            return None;
        }

        let candidate = if !self.cache.is_empty() {
            self.cache.pop_front()
        } else if self.replenish() {
            self.cache.pop_front()
        } else {
            self.translation.next()
        };

        if self.cache.is_empty() && self.translation.exhausted() {
            self.set_exhausted(true);
        }

        candidate
    }

    fn peek(&self) -> Option<Arc<dyn Candidate>> {
        if self.exhausted {
            None
        } else {
            if !self.cache.is_empty() || self.replenish() {
                self.cache.front().cloned()
            } else {
                self.translation.peek()
            }
        }
    }

    fn exhausted(&self) -> bool {
        self.exhausted
    }
}
