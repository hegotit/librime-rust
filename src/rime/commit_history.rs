use crate::rime::composition::Composition;
use crate::rime::key_event::KeyEvent;
use crate::rime::key_table::{XK_BACK_SPACE, XK_RETURN};
use crate::rime::segmentation::SegmentStatus;
use std::collections::VecDeque;

const MAX_RECORDS: usize = 20;

#[derive(Debug, Clone)]
struct CommitRecord {
    type_: String,
    text: String,
}

impl CommitRecord {
    fn new(type_: &str, text: &str) -> Self {
        Self {
            type_: type_.to_string(),
            text: text.to_string(),
        }
    }

    fn from_keycode(keycode: u32) -> Self {
        let character = if (0..=255).contains(&keycode) {
            keycode as u8 as char
        } else {
            '\0'
        };

        Self {
            type_: "thru".to_string(),
            text: character.to_string(),
        }
    }
}

#[derive(Clone)]
pub(crate) struct CommitHistory {
    records: VecDeque<CommitRecord>,
}

impl CommitHistory {
    pub(crate) fn new() -> Self {
        Self {
            records: VecDeque::new(),
        }
    }

    fn push(&mut self, record: CommitRecord) {
        self.records.push_back(record);
        if self.records.len() > MAX_RECORDS {
            self.records.pop_front();
        }
    }

    fn push_key_event(&mut self, key_event: KeyEvent) {
        if key_event.modifier() == 0 {
            match key_event.keycode() {
                XK_BACK_SPACE | XK_RETURN => self.records.clear(),
                0x20..=0x7e => self.push(CommitRecord::from_keycode(key_event.keycode())), // Printable ASCII character
                _ => {}
            }
        }
    }

    fn push_composition(&mut self, composition: &Composition, input: &str) {
        let mut last: Option<usize> = None;
        let mut end = 0;

        for seg in composition.0.segments.iter() {
            if let Some(cand) = seg.get_selected_candidate() {
                if let Some(last_index) = last {
                    let last_record = &mut self.records[last_index];
                    if last_record.type_ == cand.type_() {
                        // Join adjacent text of same type
                        last_record.text.push_str(cand.text());
                    } else {
                        // new record
                        self.push(CommitRecord::new(cand.type_(), cand.text()));
                        last = Some(self.records.len() - 1);
                    }
                } else {
                    // new record
                    self.push(CommitRecord::new(cand.type_(), cand.text()));
                    last = Some(self.records.len() - 1);
                }

                if seg.status >= SegmentStatus::Confirmed {
                    // Terminate a record by confirmation
                    last = None;
                }
                end = cand.end();
            } else {
                // No translation for the segment
                self.push(CommitRecord::new("raw", &input[seg.start..seg.end]));
                last = Some(self.records.len() - 1);
                end = seg.end;
            }
        }

        if input.len() > end {
            self.push(CommitRecord::new("raw", &input[end..]));
        }
    }

    fn repr(&self) -> String {
        self.records
            .iter()
            .map(|record| format!("[{}]{}", record.type_, record.text))
            .collect::<String>()
    }

    fn latest_text(&self) -> String {
        self.records
            .back()
            .map_or_else(String::new, |record| record.text.clone())
    }
}
