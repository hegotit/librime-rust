use std::any::Any;
use std::cmp::Ordering;
use std::collections::VecDeque;
use std::sync::Arc;

pub(crate) type CandidateQueue = VecDeque<Arc<dyn Candidate>>;
pub(crate) type CandidateList = Vec<Option<Arc<dyn Candidate>>>;

pub(crate) trait Candidate: Any {
    // Recognized by translators in learning phase
    fn type_(&self) -> &str;

    // [start, end) indicate a range in the input that corresponds to the candidate
    fn start(&self) -> usize;

    fn end(&self) -> usize;

    fn quality(&self) -> f64;

    // Candidate text to commit
    fn text(&self) -> &str;

    // (optional)
    fn comment(&self) -> &str {
        ""
    }

    // Text shown in the preedit area, replacing input string (optional)
    fn preedit(&self) -> &str {
        ""
    }

    fn set_type(&mut self, type_: &str);

    fn set_start(&mut self, start: usize);

    fn set_end(&mut self, end: usize);

    fn set_quality(&mut self, quality: f64);

    fn as_any(&self) -> &dyn Any;
}

impl Eq for dyn Candidate {}

impl PartialEq for dyn Candidate {
    fn eq(&self, other: &Self) -> bool {
        self.start() == other.start()
            && self.end() == other.end()
            && self.quality() == other.quality()
    }
}

impl Ord for dyn Candidate {
    fn cmp(&self, other: &Self) -> Ordering {
        self.start()
            .cmp(&other.start())
            .then_with(|| other.end().cmp(&self.end()))
            .then_with(|| {
                other
                    .quality()
                    .partial_cmp(&self.quality())
                    .expect("Candidate quality comparison failed")
            })
    }
}

impl PartialOrd for dyn Candidate {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub(crate) struct BaseCandidate {
    type_: String,
    start: usize,
    end: usize,
    quality: f64,
}

impl BaseCandidate {
    pub(crate) fn new(type_: String, start: usize, end: usize, quality: Option<f64>) -> Self {
        Self {
            type_,
            start,
            end,
            quality: quality.unwrap_or(0.0),
        }
    }
}

impl Candidate for BaseCandidate {
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

    fn set_type(&mut self, type_: &str) {
        self.type_ = type_.to_string();
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

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl BaseCandidate {
    fn unpack_shadow_candidate(cand: &Arc<dyn Candidate>) -> &Arc<dyn Candidate> {
        if let Some(shadow) = cand.as_any().downcast_ref::<ShadowCandidate>() {
            &shadow.item
        } else {
            cand
        }
    }

    pub(crate) fn get_genuine_candidate(cand: &Arc<dyn Candidate>) -> &Arc<dyn Candidate> {
        if let Some(uniquified) = cand.as_any().downcast_ref::<UniquifiedCandidate>() {
            Self::unpack_shadow_candidate(
                uniquified
                    .items()
                    .first()
                    .expect("Uniquified candidate is empty"),
            )
        } else {
            Self::unpack_shadow_candidate(&cand)
        }
    }

    pub(crate) fn get_genuine_candidates(cand: &Arc<dyn Candidate>) -> Vec<&Arc<dyn Candidate>> {
        if let Some(uniquified) = cand.as_any().downcast_ref::<UniquifiedCandidate>() {
            uniquified
                .items()
                .iter()
                .map(|item| Self::unpack_shadow_candidate(item))
                .collect()
        } else {
            vec![Self::unpack_shadow_candidate(cand)]
        }
    }
}

pub(crate) struct SimpleCandidate {
    base: BaseCandidate,
    text: String,
    comment: String,
    preedit: String,
}

impl SimpleCandidate {
    pub(crate) fn new(
        type_: String,
        start: usize,
        end: usize,
        text: String,
        comment: Option<String>,
        preedit: Option<String>,
    ) -> Self {
        Self {
            text,
            comment: comment.unwrap_or_default(),
            preedit: preedit.unwrap_or_default(),
            base: BaseCandidate::new(type_, start, end, None),
        }
    }

    pub(crate) fn set_text(&mut self, text: String) {
        self.text = text;
    }

    pub(crate) fn set_comment(&mut self, comment: String) {
        self.comment = comment;
    }

    pub(crate) fn set_preedit(&mut self, preedit: String) {
        self.preedit = preedit;
    }
}

impl Candidate for SimpleCandidate {
    fn type_(&self) -> &str {
        &self.base.type_
    }

    fn start(&self) -> usize {
        self.base.start
    }

    fn end(&self) -> usize {
        self.base.end
    }

    fn quality(&self) -> f64 {
        self.base.quality
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

    fn set_type(&mut self, type_: &str) {
        self.base.type_ = type_.to_string();
    }

    fn set_start(&mut self, start: usize) {
        self.base.start = start;
    }

    fn set_end(&mut self, end: usize) {
        self.base.end = end;
    }

    fn set_quality(&mut self, quality: f64) {
        self.base.quality = quality;
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub(crate) struct ShadowCandidate {
    base: BaseCandidate,
    text: String,
    comment: String,
    item: Arc<dyn Candidate>,
    inherit_comment: bool,
}

impl ShadowCandidate {
    pub(crate) fn new(
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
            base: BaseCandidate::new(type_, start, end, Some(quality)),
            text: text.unwrap_or_default(),
            comment: comment.unwrap_or_default(),
            item,
            inherit_comment: inherit_comment.unwrap_or(true),
        }
    }
}

impl Candidate for ShadowCandidate {
    fn type_(&self) -> &str {
        &self.base.type_
    }

    fn start(&self) -> usize {
        self.base.start
    }

    fn end(&self) -> usize {
        self.base.end
    }

    fn quality(&self) -> f64 {
        self.base.quality
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

    fn set_type(&mut self, type_: &str) {
        self.base.type_ = type_.to_string();
    }

    fn set_start(&mut self, start: usize) {
        self.base.start = start;
    }

    fn set_end(&mut self, end: usize) {
        self.base.end = end;
    }

    fn set_quality(&mut self, quality: f64) {
        self.base.quality = quality;
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub(crate) struct UniquifiedCandidate {
    base: BaseCandidate,
    text: String,
    comment: String,
    items: Vec<Arc<dyn Candidate>>,
}

impl UniquifiedCandidate {
    pub(crate) fn new(
        item: Arc<dyn Candidate>,
        type_: String,
        text: Option<String>,
        comment: Option<String>,
    ) -> Self {
        let start = item.start();
        let end = item.end();
        let quality = item.quality();
        Self {
            base: BaseCandidate::new(type_, start, end, Some(quality.max(0.0))),
            text: text.unwrap_or_default(),
            comment: comment.unwrap_or_default(),
            items: vec![item],
        }
    }

    fn append(&mut self, item: Arc<dyn Candidate>) {
        if self.quality() < item.quality() {
            self.set_quality(item.quality());
        }
        self.items.push(item);
    }

    fn items(&self) -> &Vec<Arc<dyn Candidate>> {
        &self.items
    }

    fn get_value<'a, F>(&'a self, preferred_value: &'a str, f: F) -> &str
    where
        F: Fn(&dyn Candidate) -> &str,
    {
        if preferred_value.is_empty() {
            self.items.first().map_or("", |item| f(item.as_ref()))
        } else {
            preferred_value
        }
    }
}

impl Candidate for UniquifiedCandidate {
    fn type_(&self) -> &str {
        &self.base.type_
    }

    fn start(&self) -> usize {
        self.base.start
    }

    fn end(&self) -> usize {
        self.base.end
    }

    fn quality(&self) -> f64 {
        self.base.quality
    }

    fn text(&self) -> &str {
        self.get_value(&self.text, |item| item.text())
    }

    fn comment(&self) -> &str {
        self.get_value(&self.comment, |item| item.comment())
    }

    fn preedit(&self) -> &str {
        self.get_value("", |item| item.preedit())
    }

    fn set_type(&mut self, type_: &str) {
        self.base.type_ = type_.to_string();
    }

    fn set_start(&mut self, start: usize) {
        self.base.start = start;
    }

    fn set_end(&mut self, end: usize) {
        self.base.end = end;
    }

    fn set_quality(&mut self, quality: f64) {
        self.base.quality = quality;
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
