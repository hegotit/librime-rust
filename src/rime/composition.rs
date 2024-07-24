use std::ops::{Deref, DerefMut};

use crate::rime::segmentation::{SegmentStatus, Segmentation};

#[derive(Default)]
pub(crate) struct Preedit {
    text: String,
    caret_pos: usize,
    sel_start: usize,
    sel_end: usize,
}

#[derive(Default, Clone)]
pub(crate) struct Composition(pub(crate) Segmentation);

impl Deref for Composition {
    type Target = Segmentation;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Composition {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Composition {
    fn has_finished_composition(&self) -> bool {
        if self.0.segments.is_empty() {
            return false;
        }
        let k = self.0.segments.len() - 1;
        if k > 0 && self.0.segments[k].start == self.0.segments[k].end {
            return self.0.segments[k - 1].status >= SegmentStatus::Selected;
        }
        self.0.segments[k].status >= SegmentStatus::Selected
    }

    pub(crate) fn get_preedit(&self, full_input: &str, caret_pos: usize, caret: &str) -> Preedit {
        let mut preedit = Preedit::default();
        preedit.caret_pos = usize::MAX;
        let mut end = 0;

        for (i, segment) in self.0.segments.iter().enumerate() {
            let start = end;
            if caret_pos == start {
                preedit.caret_pos = preedit.text.len();
            }
            let cand = segment.get_selected_candidate();
            if i < self.0.segments.len() - 1 {
                // converted
                if let Some(cand) = cand {
                    end = cand.end();
                    preedit.text.push_str(cand.text());
                } else {
                    // raw input
                    end = segment.end;
                    if !segment.has_tag("phony") {
                        preedit.text.push_str(&self.0.input[start..end]);
                    }
                }
            } else {
                // highlighted
                preedit.sel_start = preedit.text.len();
                preedit.sel_end = usize::MAX;
                if let Some(cand) = cand {
                    if !cand.preedit().is_empty() {
                        end = cand.end();
                        if let Some(caret_placeholder) = cand.preedit().find('\t') {
                            preedit.text.push_str(&cand.preedit()[..caret_placeholder]);
                            // The part after caret is considered prompt string,
                            // show it only when the caret is at the end of input.
                            if caret_pos == end && end == full_input.len() {
                                preedit.sel_end = preedit.sel_start + caret_placeholder;
                                preedit.caret_pos = preedit.sel_end;
                                preedit
                                    .text
                                    .push_str(&cand.preedit()[caret_placeholder + 1..]);
                            }
                        } else {
                            preedit.text.push_str(cand.preedit());
                        }
                    } else {
                        end = segment.end;
                        preedit.text.push_str(&self.0.input[start..end]);
                    }
                } else {
                    end = segment.end;
                    preedit.text.push_str(&self.0.input[start..end]);
                }

                if preedit.sel_end == usize::MAX {
                    preedit.sel_end = preedit.text.len();
                }
            }
        }
        if end < self.0.input.len() {
            preedit.text.push_str(&self.0.input[end..]);
            end = self.0.input.len();
        }
        if preedit.caret_pos == usize::MAX {
            preedit.caret_pos = preedit.text.len();
        }
        if end < full_input.len() {
            preedit.text.push_str(&full_input[end..]);
        }
        // Insert soft cursor and prompt string
        let prompt = format!("{}{}", caret, self.get_prompt());
        if !prompt.is_empty() {
            preedit.text.insert_str(preedit.caret_pos, &prompt);
            if preedit.caret_pos < preedit.sel_start {
                preedit.sel_start += prompt.len();
            }
            if preedit.caret_pos < preedit.sel_end {
                preedit.sel_end += prompt.len();
            }
        }
        preedit
    }

    fn get_prompt(&self) -> String {
        if self.0.segments.is_empty() {
            String::new()
        } else {
            self.0.segments.last().unwrap().prompt.clone()
        }
    }

    pub(crate) fn get_commit_text(&self) -> String {
        let mut result = String::new();
        let mut end = 0;
        for seg in &self.0.segments {
            if let Some(cand) = seg.get_selected_candidate() {
                end = cand.end();
                result.push_str(cand.text());
            } else {
                end = seg.end;
                if !seg.has_tag("phony") {
                    result.push_str(&self.0.input[seg.start..seg.end]);
                }
            }
        }
        if self.0.input.len() > end {
            result.push_str(&self.0.input[end..]);
        }
        result
    }

    pub(crate) fn get_script_text(&self) -> String {
        let mut result = String::new();
        let mut end = 0;
        for seg in &self.0.segments {
            let cand = seg.get_selected_candidate();
            let start = end;
            end = cand.as_ref().map_or(seg.end, |c| c.end());

            let text = if let Some(cand) = cand {
                if !cand.preedit().is_empty() {
                    &cand.preedit().replace("\t", "")
                } else {
                    &self.0.input[start..end]
                }
            } else {
                &self.0.input[start..end]
            };

            result.push_str(text);
        }
        if self.0.input.len() > end {
            result.push_str(&self.0.input[end..]);
        }
        result
    }

    pub(crate) fn get_debug_text(&self) -> String {
        let mut result = String::new();
        for (i, seg) in self.0.segments.iter().enumerate() {
            if i > 0 {
                result.push('|');
            }
            if !seg.tags.is_empty() {
                result.push('{');
                for (j, tag) in seg.tags.iter().enumerate() {
                    if j > 0 {
                        result.push(',');
                    }
                    result.push_str(tag);
                }
                result.push('}');
            }
            result.push_str(&self.0.input[seg.start..seg.end]);
            if let Some(cand) = seg.get_selected_candidate() {
                result.push_str("=>");
                result.push_str(cand.text());
            }
        }
        result
    }

    // Returns text of the last segment before the given position.
    fn get_text_before(&self, pos: usize) -> String {
        for seg in self.0.segments.iter().rev() {
            if seg.end <= pos {
                if let Some(cand) = seg.get_selected_candidate() {
                    return cand.text().to_string();
                }
            }
        }
        String::new()
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.segments.is_empty()
    }
}
