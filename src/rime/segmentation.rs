use std::collections::HashSet;
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::sync::{Arc, RwLock};

use log::{error, info};

use crate::rime::candidate::Candidate;
use crate::rime::menu::Menu;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Default, PartialOrd)]
pub(crate) enum SegmentStatus {
    #[default]
    Void,
    Guess,
    Selected,
    Confirmed,
}

#[derive(Default, Clone)]
pub(crate) struct Segment {
    pub(crate) status: SegmentStatus,
    pub(crate) start: usize,
    pub(crate) end: usize,
    length: usize,
    pub(crate) tags: HashSet<String>,
    pub(crate) menu: Option<Arc<RwLock<Menu>>>,
    pub(crate) selected_index: usize,
    pub(crate) prompt: String,
}

impl Segment {
    fn new(start_pos: usize, end_pos: usize) -> Self {
        Self {
            start: start_pos,
            end: end_pos,
            length: end_pos - start_pos,
            ..Default::default()
        }
    }

    fn clear(&mut self) {
        self.status = SegmentStatus::Void;
        self.tags.clear();
        self.menu = None;
        self.selected_index = 0;
        self.prompt.clear();
    }

    fn close(&mut self) {
        if let Some(cand) = self.get_selected_candidate() {
            if cand.end() < self.end {
                // having selected a partially matched candidate, split it into 2 segments
                self.end = cand.end();
                self.tags.insert(String::from("partial"));
            }
        }
    }

    pub(crate) fn reopen(&mut self, caret_pos: usize) -> bool {
        if self.status < SegmentStatus::Selected {
            return false;
        }
        let original_end_pos = self.start + self.length;
        if original_end_pos == caret_pos {
            // reuse previous candidates and keep selection
            if self.end < original_end_pos {
                // restore partial-selected segment
                self.end = original_end_pos;
                self.tags.remove("partial");
            }
            self.status = SegmentStatus::Guess;
        } else {
            self.status = SegmentStatus::Void;
        }
        true
    }

    pub(crate) fn has_tag(&self, tag: &str) -> bool {
        self.tags.contains(tag)
    }

    pub(crate) fn get_candidate_at(&self, index: usize) -> Option<Arc<dyn Candidate>> {
        match self.menu.as_ref()?.write() {
            Ok(mut writable) => writable.get_candidate_at(index),
            Err(e) => {
                error!("Failed to acquire write lock: {:?}", e);
                None
            }
        }
    }

    pub(crate) fn get_selected_candidate(&self) -> Option<Arc<dyn Candidate>> {
        self.get_candidate_at(self.selected_index)
    }
}

#[derive(Default, Clone)]
pub(crate) struct Segmentation {
    pub(crate) input: String,
    pub(crate) segments: Vec<Segment>,
}

impl Segmentation {
    fn reset(&mut self, new_input: &str) {
        info!("reset to {} segments.", self.segments.len());

        let input_bytes = self.input.as_bytes();
        let new_input_bytes = new_input.as_bytes();
        // mark redo segmentation, while keeping user confirmed segments
        let mut diff_pos = 0;
        while diff_pos < self.input.len()
            && diff_pos < new_input.len()
            && input_bytes[diff_pos] == new_input_bytes[diff_pos]
        {
            diff_pos += 1;
        }

        // dispose segments that have changed
        let mut disposed = false;
        while let Some(last) = self.segments.last() {
            if last.end > diff_pos {
                self.segments.pop();
                if !disposed {
                    disposed = true;
                }
            } else {
                break;
            }
        }

        if disposed {
            self.forward();
        }

        self.input = new_input.to_string();
    }

    fn reset_segments(&mut self, num_segments: usize) {
        if num_segments < self.segments.len() {
            self.segments.truncate(num_segments);
        }
    }

    fn add_segment(&mut self, segment: Segment) -> bool {
        let start = self.get_current_start_position();
        if segment.start != start {
            // rule one: in one round, we examine only those segs
            // that are left-aligned to a same position
            return false;
        }

        if self.segments.is_empty() {
            self.segments.push(segment);
            return true;
        }

        let last = self.segments.last_mut().unwrap();
        if last.end > segment.end {
            // rule two: always prefer the longer segment...
        } else if last.end < segment.end {
            // ...and overwrite the shorter one
            *last = segment;
        } else {
            // rule three: with segments equal in length, merge their tags
            last.tags.extend(segment.tags);
        }
        true
    }

    // Finalize a round
    pub(crate) fn forward(&mut self) -> bool {
        if let Some(last) = self.segments.last() {
            if last.start == last.end {
                return false;
            }
            // initialize an empty segment for the next round
            self.segments.push(Segment::new(last.end, last.end));
            true
        } else {
            false
        }
    }

    // Remove empty trailing segment
    pub(crate) fn trim(&mut self) -> bool {
        if let Some(last) = self.segments.last() {
            if last.start == last.end {
                self.segments.pop();
                return true;
            }
        }
        false
    }

    fn has_finished_segmentation(&self) -> bool {
        self.segments.last().map_or(0, |seg| seg.end) >= self.input.len()
    }

    fn get_current_start_position(&self) -> usize {
        self.segments.last().map_or(0, |seg| seg.start)
    }

    fn get_current_end_position(&self) -> usize {
        self.segments.last().map_or(0, |seg| seg.end)
    }

    fn get_current_segment_length(&self) -> usize {
        self.segments.last().map_or(0, |seg| seg.end - seg.start)
    }

    fn get_confirmed_position(&self) -> usize {
        self.segments
            .iter()
            .filter(|seg| seg.status >= SegmentStatus::Selected)
            .map(|seg| seg.end)
            .last()
            .unwrap_or(0)
    }

    fn input(&self) -> &str {
        &self.input
    }
}

impl Display for Segmentation {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "[{}", self.input)?;
        for segment in &self.segments {
            write!(f, "|{},{}", segment.start, segment.end)?;
            if !segment.tags.is_empty() {
                write!(f, "{{")?;
                for (i, tag) in segment.tags.iter().enumerate() {
                    if i > 0 {
                        write!(f, ",")?;
                    }
                    write!(f, "{}", tag)?;
                }
                write!(f, "}}")?;
            }
        }
        write!(f, "]")
    }
}
