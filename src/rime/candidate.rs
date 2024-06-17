use std::any::Any;
use std::cmp::Ordering;
use std::collections::LinkedList;
use std::fmt::Debug;
use std::sync::Arc;

pub type CandidateQueue = LinkedList<Arc<dyn Candidate>>;

pub trait Candidate: Debug + Any {
    fn type_(&self) -> &str;
    fn start(&self) -> usize;
    fn end(&self) -> usize;
    fn quality(&self) -> f64;
    fn text(&self) -> &str;
    fn comment(&self) -> &str {
        ""
    }
    fn preedit(&self) -> &str {
        ""
    }
    fn as_any(&self) -> &dyn Any;
}

#[derive(Debug, Clone)]
pub struct BasicCandidate {
    type_: String,
    start: usize,
    end: usize,
    quality: f64,
}

impl BasicCandidate {
    pub fn new(type_: String, start: usize, end: usize, quality: f64) -> Self {
        Self {
            type_,
            start,
            end,
            quality,
        }
    }

    pub fn set_type(&mut self, type_: String) {
        self.type_ = type_;
    }

    pub fn set_start(&mut self, start: usize) {
        self.start = start;
    }

    pub fn set_end(&mut self, end: usize) {
        self.end = end;
    }

    pub fn set_quality(&mut self, quality: f64) {
        self.quality = quality;
    }
}

impl Candidate for BasicCandidate {
    fn type_(&self) -> &str {
        &self.type_
    }

    fn start(&self) -> usize {
        self.start
    }

    fn end(&self) -> usize {
        self.end
    }

    fn quality(&self) -> f64 {
        self.quality
    }

    fn text(&self) -> &str {
        ""
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl BasicCandidate {
    pub fn compare(&self, other: &dyn Candidate) -> Ordering {
        self.start
            .cmp(&other.start())
            .then_with(|| other.end().cmp(&self.end))
            .then_with(|| {
                other
                    .quality()
                    .partial_cmp(&self.quality)
                    .unwrap_or(Ordering::Equal)
            })
    }

    pub fn get_genuine_candidate(cand: &Arc<dyn Candidate>) -> &Arc<dyn Candidate> {
        if let Some(uniquified) = cand.as_any().downcast_ref::<UniquifiedCandidate>() {
            unpack_shadow_candidate(uniquified.items().first().unwrap())
        } else {
            unpack_shadow_candidate(&cand)
        }
    }

    pub fn get_genuine_candidates(cand: &Arc<dyn Candidate>) -> Vec<&Arc<dyn Candidate>> {
        let mut result = vec![];
        if let Some(uniquified) = cand.as_any().downcast_ref::<UniquifiedCandidate>() {
            for item in uniquified.items() {
                result.push(unpack_shadow_candidate(item));
            }
        } else {
            result.push(unpack_shadow_candidate(&cand));
        }
        result
    }
}

fn unpack_shadow_candidate(cand: &Arc<dyn Candidate>) -> &Arc<dyn Candidate> {
    if let Some(shadow) = cand.as_any().downcast_ref::<ShadowCandidate>() {
        &shadow.item
    } else {
        cand
    }
}

#[derive(Debug, Clone)]
pub struct SimpleCandidate {
    type_: String,
    start: usize,
    end: usize,
    quality: f64,
    text: String,
    comment: String,
    preedit: String,
}

impl SimpleCandidate {
    pub fn new(
        type_: String,
        start: usize,
        end: usize,
        text: String,
        comment: Option<String>,
        preedit: Option<String>,
    ) -> Self {
        Self {
            type_,
            start,
            end,
            quality: 0.0,
            text,
            comment: comment.unwrap_or_default(),
            preedit: preedit.unwrap_or_default(),
        }
    }

    pub fn set_type(&mut self, type_: String) {
        self.type_ = type_;
    }

    pub fn set_start(&mut self, start: usize) {
        self.start = start;
    }

    pub fn set_end(&mut self, end: usize) {
        self.end = end;
    }

    pub fn set_quality(&mut self, quality: f64) {
        self.quality = quality;
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

impl Candidate for SimpleCandidate {
    fn type_(&self) -> &str {
        &self.type_
    }

    fn start(&self) -> usize {
        self.start
    }

    fn end(&self) -> usize {
        self.end
    }

    fn quality(&self) -> f64 {
        self.quality
    }

    fn text(&self) -> &str {
        &self.text
    }

    fn comment(&self) -> &str {
        &self.comment
    }

    fn preedit(&self) -> &str {
        &self.preedit
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Debug, Clone)]
pub struct ShadowCandidate {
    type_: String,
    start: usize,
    end: usize,
    quality: f64,
    text: String,
    comment: String,
    item: Arc<dyn Candidate>,
    inherit_comment: bool,
}

impl ShadowCandidate {
    pub fn new(
        item: Arc<dyn Candidate>,
        type_: String,
        text: Option<String>,
        comment: Option<String>,
        inherit_comment: Option<bool>,
    ) -> Self {
        let start = item.start();
        let end = item.end();
        let quality = item.quality();
        Self {
            type_,
            start,
            end,
            quality,
            text: text.unwrap_or_default(),
            comment: comment.unwrap_or_default(),
            item,
            inherit_comment: inherit_comment.unwrap_or(true),
        }
    }
    pub fn set_type(&mut self, type_: String) {
        self.type_ = type_;
    }

    pub fn set_start(&mut self, start: usize) {
        self.start = start;
    }

    pub fn set_end(&mut self, end: usize) {
        self.end = end;
    }

    pub fn set_quality(&mut self, quality: f64) {
        self.quality = quality;
    }
}

impl Candidate for ShadowCandidate {
    fn type_(&self) -> &str {
        &self.type_
    }

    fn start(&self) -> usize {
        self.start
    }

    fn end(&self) -> usize {
        self.end
    }

    fn quality(&self) -> f64 {
        self.quality
    }

    fn text(&self) -> &str {
        if self.text.is_empty() {
            self.item.text()
        } else {
            &self.text
        }
    }

    fn comment(&self) -> &str {
        if self.inherit_comment && self.comment.is_empty() {
            self.item.comment()
        } else {
            &self.comment
        }
    }

    fn preedit(&self) -> &str {
        self.item.preedit()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Debug, Clone)]
pub struct UniquifiedCandidate {
    type_: String,
    start: usize,
    end: usize,
    quality: f64,
    text: String,
    comment: String,
    items: Vec<Arc<dyn Candidate>>,
}

impl UniquifiedCandidate {
    pub fn new(
        item: Arc<dyn Candidate>,
        type_: String,
        text: Option<String>,
        comment: Option<String>,
    ) -> Self {
        let start = item.start();
        let end = item.end();
        let quality = item.quality();
        let mut candidate = Self {
            type_,
            start,
            end,
            quality,
            text: text.unwrap_or_default(),
            comment: comment.unwrap_or_default(),
            items: vec![],
        };
        candidate.append(item);
        candidate
    }

    pub fn append(&mut self, item: Arc<dyn Candidate>) {
        if self.quality() < item.quality() {
            self.set_quality(item.quality());
        }
        self.items.push(item);
    }

    pub fn set_type(&mut self, type_: String) {
        self.type_ = type_;
    }

    pub fn set_start(&mut self, start: usize) {
        self.start = start;
    }

    pub fn set_end(&mut self, end: usize) {
        self.end = end;
    }

    pub fn set_quality(&mut self, quality: f64) {
        self.quality = quality;
    }

    fn items(&self) -> &Vec<Arc<dyn Candidate>> {
        &self.items
    }

    fn get_field_or_first<'a, F>(&'a self, field: &'a str, f: F) -> &str
    where
        F: Fn(&dyn Candidate) -> &str,
    {
        if field.is_empty() {
            self.items.first().map_or("", |item| f(item.as_ref()))
        } else {
            field
        }
    }
}

impl Candidate for UniquifiedCandidate {
    fn type_(&self) -> &str {
        &self.type_
    }

    fn start(&self) -> usize {
        self.start
    }

    fn end(&self) -> usize {
        self.end
    }

    fn quality(&self) -> f64 {
        self.quality
    }

    fn text(&self) -> &str {
        self.get_field_or_first(&self.text, |item| item.text())
    }

    fn comment(&self) -> &str {
        self.get_field_or_first(&self.comment, |item| item.comment())
    }

    fn preedit(&self) -> &str {
        self.get_field_or_first("", |item| item.preedit())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
