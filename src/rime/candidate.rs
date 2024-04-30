use std::cell::RefCell;
use std::cmp::Ordering;
use std::rc::Rc;
use std::sync::Arc;

pub type An<T> = Arc<T>;
pub type Of<T> = An<T>;
pub type Weak<T> = std::sync::Weak<T>;

pub fn as_cast<X, Y>(ptr: &An<Y>) -> Option<An<X>>
where
    X: 'static + ?Sized,
    Y: 'static + ?Sized,
{
    Arc::downcast(Arc::clone(ptr)).ok()
}

pub fn is_instance<X, Y>(ptr: &An<Y>) -> bool
where
    X: 'static + ?Sized,
    Y: 'static + ?Sized,
{
    as_cast::<X, Y>(ptr).is_some()
}

pub fn new_instance<T, Args>(args: Args) -> An<T>
where
    T: 'static + ?Sized,
    Args: std::convert::Into<Arc<T>>,
{
    Arc::new(args.into())
}

pub trait Candidate: Sync + Send {
    fn get_type(&self) -> &str;
    fn get_start(&self) -> usize;
    fn get_end(&self) -> usize;
    fn get_quality(&self) -> f64;
    fn get_text(&self) -> &str;
    fn get_comment(&self) -> String;
    fn get_preedit(&self) -> String;

    fn compare(&self, other: &dyn Candidate) -> Ordering {
        let start_cmp = self.get_start().cmp(&other.get_start());
        if start_cmp != Ordering::Equal {
            return start_cmp;
        }
        let end_cmp = other.get_end().cmp(&self.get_end());
        if end_cmp != Ordering::Equal {
            return end_cmp;
        }
        self.get_quality()
            .partial_cmp(&other.get_quality())
            .unwrap_or(Ordering::Equal)
            .reverse()
    }

    fn set_type(&mut self, _type: String);
    fn set_start(&mut self, start: usize);
    fn set_end(&mut self, end: usize);
    fn set_quality(&mut self, quality: f64);
}

pub struct SimpleCandidate {
    _type: String,
    start: usize,
    end: usize,
    quality: f64,
    text: String,
    comment: String,
    preedit: String,
}

impl Candidate for SimpleCandidate {
    fn get_type(&self) -> &str {
        &self._type
    }
    fn get_start(&self) -> usize {
        self.start
    }
    fn get_end(&self) -> usize {
        self.end
    }
    fn get_quality(&self) -> f64 {
        self.quality
    }
    fn get_text(&self) -> &str {
        &self.text
    }
    fn get_comment(&self) -> String {
        self.comment.clone()
    }
    fn get_preedit(&self) -> String {
        self.preedit.clone()
    }

    fn set_type(&mut self, _type: String) {
        self._type = _type;
    }

    fn set_start(&mut self, start: usize) {
        self.start = start;
    }
    fn set_end(&mut self, end: usize) {
        self.end = end;
    }
    fn set_quality(&mut self, quality: f64) {
        self.quality = quality;
    }
}

impl SimpleCandidate {
    pub fn new() -> Self {
        SimpleCandidate {
            _type: String::new(),
            start: 0,
            end: 0,
            quality: 0.0,
            text: String::new(),
            comment: String::new(),
            preedit: String::new(),
        }
    }

    pub fn new_with_params(
        _type: String,
        start: usize,
        end: usize,
        text: String,
        comment: Option<String>,
        preedit: Option<String>,
    ) -> Self {
        Self {
            _type,
            start,
            end,
            quality: 0.0,
            text,
            comment: comment.unwrap_or_default(),
            preedit: preedit.unwrap_or_default(),
        }
    }

    pub fn set_text(&mut self, text: String) {
        self.text = text;
    }

    pub fn set_comment(&mut self, comment: String) {
        self.comment = comment;
    }

    pub fn set_preedit(&mut self, preedit: String) {
        self.preedit = preedit;
    }
}

pub struct ShadowCandidate {
    item: An<dyn Candidate>,
    _type: String,
    text: String,
    comment: String,
    inherit_comment: bool,
}

impl Candidate for ShadowCandidate {
    fn get_type(&self) -> &str {
        &self._type
    }
    fn get_start(&self) -> usize {
        self.item.get_start()
    }
    fn get_end(&self) -> usize {
        self.item.get_end()
    }
    fn get_quality(&self) -> f64 {
        self.item.get_quality()
    }
    fn get_text(&self) -> &str {
        if self.text.is_empty() {
            self.item.get_text()
        } else {
            &self.text
        }
    }
    fn get_comment(&self) -> String {
        if self.inherit_comment && self.comment.is_empty() {
            self.item.get_comment()
        } else {
            self.comment.clone()
        }
    }
    fn get_preedit(&self) -> String {
        self.item.get_preedit()
    }

    fn set_type(&mut self, _type: String) {
        self._type = _type;
    }
    fn set_start(&mut self, _start: usize) {
        // Do nothing
    }
    fn set_end(&mut self, _end: usize) {
        // Do nothing
    }
    fn set_quality(&mut self, _quality: f64) {
        // Do nothing
    }
}

pub struct UniquifiedCandidate {
    items: CandidateList,
    _type: String,
    text: String,
    comment: String,
    quality: f64,
}

impl Candidate for UniquifiedCandidate {
    fn get_type(&self) -> &str {
        &self._type
    }
    fn get_start(&self) -> usize {
        self.items.first().unwrap().get_start()
    }
    fn get_end(&self) -> usize {
        self.items.first().unwrap().get_end()
    }
    fn get_quality(&self) -> f64 {
        self.quality
    }
    fn get_text(&self) -> &str {
        if self.text.is_empty() {
            self.items.first().unwrap().get_text()
        } else {
            &self.text
        }
    }
    fn get_comment(&self) -> String {
        if self.comment.is_empty() {
            self.items.first().unwrap().get_comment()
        } else {
            self.comment.clone()
        }
    }
    fn get_preedit(&self) -> String {
        self.items.first().unwrap().get_preedit()
    }

    fn set_type(&mut self, _type: String) {
        self._type = _type;
    }
    fn set_start(&mut self, _start: usize) {
        // Do nothing
    }
    fn set_end(&mut self, _end: usize) {
        // Do nothing
    }
    fn set_quality(&mut self, quality: f64) {
        self.quality = quality;
    }
}

impl UniquifiedCandidate {
    pub fn append(&mut self, item: An<dyn Candidate>) {
        self.items.push(item.clone());
        if self.get_quality() < item.get_quality() {
            self.set_quality(item.get_quality());
        }
    }

    pub fn get_items(&self) -> &CandidateList {
        &self.items
    }
}

pub fn get_genuine_candidate(cand: &An<dyn Candidate>) -> An<dyn Candidate> {
    if let Some(uniquified) = as_cast::<UniquifiedCandidate, _>(cand) {
        unpack_shadow_candidate(uniquified.get_items().first().unwrap())
    } else {
        unpack_shadow_candidate(cand)
    }
}

pub fn get_genuine_candidates(cand: &An<dyn Candidate>) -> CandidateList {
    let mut result = vec![];
    if let Some(uniquified) = as_cast::<UniquifiedCandidate, _>(cand) {
        for item in uniquified.get_items() {
            result.push(unpack_shadow_candidate(item));
        }
    } else {
        result.push(unpack_shadow_candidate(cand));
    }
    result
}

fn unpack_shadow_candidate(cand: &An<dyn Candidate>) -> An<dyn Candidate> {
    if let Some(shadow) = as_cast::<ShadowCandidate, _>(cand) {
        Arc::clone(&shadow.item)
    } else {
        Arc::clone(cand)
    }
}
