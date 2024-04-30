use std::collections::VecDeque;

const K_MAX_RECORDS: usize = 20;

#[derive(Debug, Clone)]
struct CommitRecord {
    record_type: String,
    text: String,
}

impl CommitRecord {
    fn new(record_type: &str, text: &str) -> Self {
        Self {
            record_type: record_type.to_string(),
            text: text.to_string(),
        }
    }

    fn from_keycode(keycode: u8) -> Self {
        Self {
            record_type: "thru".to_string(),
            text: (keycode as char).to_string(),
        }
    }
}

struct KeyEvent {
    modifier: u32,
    keycode: u8,
}

impl KeyEvent {
    fn new(modifier: u32, keycode: u8) -> Self {
        Self { modifier, keycode }
    }

    fn modifier(&self) -> u32 {
        self.modifier
    }

    fn keycode(&self) -> u8 {
        self.keycode
    }
}

struct Segment {
    start: usize,
    end: usize,
    status: SegmentStatus,
    selected_candidate: Option<Candidate>,
}

enum SegmentStatus {
    Pending,
    Confirmed,
}

impl Segment {
    fn new(start: usize, end: usize, status: SegmentStatus, selected_candidate: Option<Candidate>) -> Self {
        Self {
            start,
            end,
            status,
            selected_candidate,
        }
    }

    fn start(&self) -> usize {
        self.start
    }

    fn end(&self) -> usize {
        self.end
    }

    fn status(&self) -> &SegmentStatus {
        &self.status
    }

    fn get_selected_candidate(&self) -> Option<&Candidate> {
        self.selected_candidate.as_ref()
    }
}

struct Candidate {
    record_type: String,
    text: String,
    end: usize,
}


impl Candidate {
    fn new(record_type: &str, text: &str, end: usize) -> Self {
        Self {
            record_type: record_type.to_string(),
            text: text.to_string(),
            end,
        }
    }

    fn type_(&self) -> &str {
        &self.record_type
    }

    fn text(&self) -> &str {
        &self.text
    }

    fn end(&self) -> usize {
        self.end
    }
}

struct Composition {
    segments: Vec<Segment>,
}

impl Composition {
    fn new(segments: Vec<Segment>) -> Self {
        Self { segments }
    }

    fn iter(&self) -> std::slice::Iter<Segment> {
        self.segments.iter()
    }
}

struct CommitHistory {
    records: VecDeque<CommitRecord>,
}

impl CommitHistory {
    fn new() -> Self {
        Self {
            records: VecDeque::new(),
        }
    }

    fn push(&mut self, record: CommitRecord) {
        self.records.push_back(record);
        if self.records.len() > K_MAX_RECORDS {
            self.records.pop_front();
        }
    }

    fn push_key_event(&mut self, key_event: KeyEvent) {
        if key_event.modifier() == 0 {
            match key_event.keycode() {
                8 | 13 => self.records.clear(), // BackSpace or Return
                32..=126 => self.push(CommitRecord::from_keycode(key_event.keycode())), // printable ASCII characters
                _ => {}
            }
        }
    }


    fn push_composition(&mut self, composition: &Composition, input: &str) {
        let mut last: Option<&mut CommitRecord> = None;
        let mut end = 0;

        for seg in composition.iter() {
            if let Some(cand) = seg.get_selected_candidate() {
                if let Some(last_record) = &mut last {
                    if last_record.record_type == cand.type_() {
                        last_record.text.push_str(cand.text());
                    } else {
                        self.push(CommitRecord::new(cand.type_(), cand.text()));
                        last = Some(self.records.back_mut().unwrap());
                    }
                } else {
                    self.push(CommitRecord::new(cand.type_(), cand.text()));
                    last = Some(self.records.back_mut().unwrap());
                }


                if matches!(seg.status(), SegmentStatus::Confirmed) {
                    last = None;
                }
                end = cand.end();
            } else {
                self.push(CommitRecord::new("raw", &input[seg.start()..seg.end()]));
                end = seg.end();
            }
        }

        if input.len() > end {
            self.push(CommitRecord::new("raw", &input[end..]));
        }
    }

    fn repr(&self) -> String {
        self.records
            .iter()
            .map(|record| format!("[{}]{}", record.record_type, record.text))
            .collect::<String>()
    }

    fn latest_text(&self) -> String {
        self.records.back().map_or_else(String::new, |record| record.text.clone())
    }
}