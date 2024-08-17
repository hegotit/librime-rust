use crate::rime::candidate::Candidate;
use crate::rime::commit_history::CommitHistory;
use crate::rime::composition::{Composition, Preedit};
use crate::rime::key_event::KeyEvent;
use crate::rime::segmentation::SegmentStatus;
use log::info;
use signals2::{Emit1, Emit2, Signal};
use std::cmp::min;
use std::collections::BTreeMap;
use std::sync::Arc;

type Notifier = Signal<(Arc<Context>,)>;
type OptionUpdateNotifier = Signal<(Arc<Context>, String)>;
type PropertyUpdateNotifier = Signal<(Arc<Context>, String)>;
type KeyEventNotifier = Signal<(Arc<Context>, KeyEvent)>;

#[derive(Clone)]
struct Context {
    input: String,
    caret_pos: usize,
    composition: Composition,
    commit_history: CommitHistory,
    options: BTreeMap<String, bool>,
    properties: BTreeMap<String, String>,
    commit_notifier: Notifier,
    select_notifier: Notifier,
    update_notifier: Notifier,
    delete_notifier: Notifier,
    option_update_notifier: OptionUpdateNotifier,
    property_update_notifier: PropertyUpdateNotifier,
    unhandled_key_notifier: KeyEventNotifier,
}

impl Context {
    fn new() -> Self {
        Self {
            input: String::new(),
            caret_pos: 0,
            composition: Composition::default(),
            commit_history: CommitHistory::new(),
            options: BTreeMap::new(),
            properties: BTreeMap::new(),
            commit_notifier: Notifier::new(),
            select_notifier: Notifier::new(),
            update_notifier: Notifier::new(),
            delete_notifier: Notifier::new(),
            option_update_notifier: OptionUpdateNotifier::new(),
            property_update_notifier: PropertyUpdateNotifier::new(),
            unhandled_key_notifier: KeyEventNotifier::new(),
        }
    }

    fn commit(&mut self) -> bool {
        if !self.is_composing() {
            return false;
        }
        let context = Arc::new(self.clone());
        // Notify the engine and interested components
        {
            // self.commit_notifier.emit(context.clone());
            todo!()
        }
        // start over
        self.clear();
        true
    }

    fn get_commit_text(&self) -> String {
        if self.get_option("dumb") {
            return String::new();
        }
        self.composition.get_commit_text()
    }

    fn get_script_text(&self) -> String {
        self.composition.get_script_text()
    }

    fn get_preedit(&self) -> Preedit {
        self.composition
            .get_preedit(&self.input, self.caret_pos, &self.get_soft_cursor())
    }

    fn is_composing(&self) -> bool {
        !self.input.is_empty() || !self.composition.is_empty()
    }

    fn has_menu(&self) -> bool {
        if self.composition.is_empty() {
            return false;
        }
        if let Some(menu) = &self.composition.segments.last().unwrap().menu {
            !menu.read().unwrap().is_empty()
        } else {
            false
        }
    }

    fn get_selected_candidate(&self) -> Option<Arc<dyn Candidate>> {
        if self.composition.is_empty() {
            return None;
        }
        self.composition.segments.last()?.get_selected_candidate()
    }

    fn push_input(&mut self, ch: char) -> bool {
        if self.caret_pos >= self.input.len() {
            self.input.push(ch);
            self.caret_pos = self.input.len();
        } else {
            self.input.insert(self.caret_pos, ch);
            self.caret_pos += 1;
        }
        {
            // self.update_notifier.emit(self.clone());
            todo!()
        }
        true
    }

    fn push_input_str(&mut self, str: &str) -> bool {
        if self.caret_pos >= self.input.len() {
            self.input.push_str(str);
            self.caret_pos = self.input.len();
        } else {
            self.input.insert_str(self.caret_pos, str);
            self.caret_pos += str.len();
        }
        {
            // self.update_notifier.emit(self.clone());
            todo!()
        }
        true
    }

    fn pop_input(&mut self, len: usize) -> bool {
        if self.caret_pos < len {
            return false;
        }
        self.caret_pos -= len;
        self.input.drain(self.caret_pos..self.caret_pos + len);
        {
            //   self.update_notifier.emit(self.clone());
            todo!()
        }
        true
    }

    fn delete_input(&mut self, len: usize) -> bool {
        if self.caret_pos + len > self.input.len() {
            return false;
        }
        self.input.drain(self.caret_pos..self.caret_pos + len);
        {
            // self.update_notifier.emit(self.clone());
            todo!()
        }
        true
    }

    fn clear(&mut self) {
        self.input.clear();
        self.caret_pos = 0;
        self.composition.segments.clear();
        {
            // self.update_notifier.emit(self.clone());
            todo!()
        }
    }

    // Return false if there is no candidate at index
    fn select(&mut self, index: usize) -> bool {
        if self.composition.is_empty() {
            return false;
        }
        let seg = self.composition.segments.last_mut();
        if let Some(seg) = seg {
            if let Some(cand) = seg.get_candidate_at(index) {
                seg.selected_index = index;
                seg.status = SegmentStatus::Selected;
                info!("Selected: '{}', index = {}", cand.text(), index);
                {
                    // self.select_notifier.emit(self.clone());
                    todo!()
                }
            }
            true
        } else {
            false
        }
    }

    // Return false if the selected index has not changed
    fn highlight(&mut self, index: usize) -> bool {
        if self.composition.is_empty() || self.composition.segments.last().unwrap().menu.is_none() {
            return false;
        }
        let seg = self.composition.segments.last_mut().unwrap();
        let candidate_count = seg
            .menu
            .as_ref()
            .unwrap()
            .write()
            .unwrap()
            .prepare(index + 1);

        let new_index = if candidate_count > 0 {
            min(candidate_count - 1, index)
        } else {
            0
        };
        let previous_index = seg.selected_index;
        if previous_index == new_index {
            info!("Selection has not changed, currently at {}", new_index);
            return false;
        }
        seg.selected_index = new_index;
        {
            // self.update_notifier.emit(self.clone());
            todo!()
        }
        info!(
            "Selection changed from: {} to: {}",
            previous_index, new_index
        );
        true
    }

    fn delete_candidate_by_index(&mut self, index: usize) -> bool {
        if let Some(seg) = self.composition.segments.last() {
            if let Some(candidate) = seg.get_candidate_at(index) {
                return self.delete_candidate(Some(candidate));
            }
        }
        false
    }

    fn delete_current_selection(&mut self) -> bool {
        if let Some(seg) = self.composition.segments.last() {
            if let Some(candidate) = seg.get_selected_candidate() {
                return self.delete_candidate(Some(candidate));
            }
        }
        false
    }

    // Return false if there's no candidate for current segment
    fn confirm_current_selection(&mut self) -> bool {
        if self.composition.is_empty() {
            return false;
        }
        if let Some(seg) = self.composition.segments.last_mut() {
            seg.status = SegmentStatus::Selected;
            if let Some(cand) = seg.get_selected_candidate() {
                info!(
                    "Confirmed: '{}', selected_index = {}",
                    cand.text(),
                    seg.selected_index
                );
            } else {
                if seg.end == seg.start {
                    // fluid_editor will confirm the whole sentence
                    return false;
                }
                // confirm raw input
            }
        }

        {
            // self.select_notifier.emit(self.clone());
            todo!()
        }
        true
    }

    fn begin_editing(&mut self) {
        for seg in self.composition.segments.iter_mut().rev() {
            if seg.status > SegmentStatus::Selected {
                return;
            }
            if seg.status == SegmentStatus::Selected {
                seg.tags.insert("selected_before_editing".to_string());
                return;
            }
        }
    }

    fn reopen_previous_segment(&mut self) -> bool {
        if self.composition.trim() {
            if !self.composition.is_empty() {
                if let Some(seg) = self.composition.segments.last_mut() {
                    if seg.status >= SegmentStatus::Selected {
                        seg.reopen(self.caret_pos);
                    }
                }
            }
            {
                // self.update_notifier.emit(self.clone());
                todo!()
            }
            true
        } else {
            false
        }
    }

    fn clear_previous_segment(&mut self) -> bool {
        if let Some(last_segment) = self.composition.segments.last() {
            let where_ = last_segment.start;
            if where_ < self.input.len() {
                self.set_input(self.input[..where_].to_string());
                return true;
            }
        }
        false
    }

    fn reopen_previous_selection(&mut self) -> bool {
        let len = self.composition.segments.len();
        let mut target_index = None;

        for i in (0..len).rev() {
            let seg = &mut self.composition.segments[i];
            if seg.status > SegmentStatus::Selected {
                return false;
            }
            if seg.status == SegmentStatus::Selected {
                // Do not reopen the previous selection after editing input
                if seg.tags.contains("selected_before_editing") {
                    return false;
                }

                target_index = Some(i);
                break;
            }
        }

        if let Some(index) = target_index {
            self.composition.segments.truncate(index + 1);
            self.composition.segments[index].reopen(self.caret_pos);
            {
                // self.update_notifier.emit(self.clone());
                todo!()
            }
            return true;
        }

        false
    }

    fn clear_non_confirmed_composition(&mut self) -> bool {
        if self.composition.segments.is_empty() {
            return false;
        }

        if let Some(pos) = self
            .composition
            .segments
            .iter()
            .rposition(|seg| seg.status >= SegmentStatus::Selected)
        {
            self.composition.segments.truncate(pos + 1);
        } else {
            self.composition.segments.clear();
        }

        self.composition.forward();
        info!("composition: {}", self.composition.get_debug_text());

        true
    }

    fn refresh_non_confirmed_composition(&mut self) -> bool {
        if self.clear_non_confirmed_composition() {
            {
                // self.update_notifier.emit(self.clone());
                todo!()
            }
            true
        } else {
            false
        }
    }

    fn set_caret_pos(&mut self, caret_pos: usize) {
        if caret_pos > self.input.len() {
            self.caret_pos = self.input.len();
        } else {
            self.caret_pos = caret_pos;
        }
        {
            // self.update_notifier.emit(self.clone());
            todo!()
        }
    }

    fn set_composition(&mut self, comp: Composition) {
        self.composition = comp;
    }

    fn set_input(&mut self, value: String) {
        self.input = value;
        self.caret_pos = self.input.len();
        {
            // self.update_notifier.emit(self.clone());
            todo!()
        }
    }

    fn set_option(&mut self, name: &str, value: bool) {
        self.options.insert(name.to_string(), value);
        info!("Context::set_option {} = {}", name, value);
        {
            /*  self.option_update_notifier.emit(self.clone(), name.to_string()); */
            todo!()
        }
    }

    fn get_option(&self, name: &str) -> bool {
        *self.options.get(name).unwrap_or(&false)
    }

    fn set_property(&mut self, name: &str, value: &str) {
        self.properties.insert(name.to_string(), value.to_string());
        {
            // self.property_update_notifier
            //     .emit(self.clone(), name.to_string());
            todo!()
        }
    }

    fn get_property(&self, name: &str) -> String {
        self.properties.get(name).cloned().unwrap_or_default()
    }

    fn clear_transient_options(&mut self) {
        info!("Context::clear_transient_options");
        let opt_keys = self
            .options
            .range("_".to_string()..)
            .filter(|(k, _)| k.starts_with('_'))
            .map(|(k, _)| k.clone())
            .collect::<Vec<_>>();

        for k in opt_keys {
            info!("Cleared option: {}", k);
            self.options.remove(&k);
        }

        let prop_keys = self
            .properties
            .range("_".to_string()..)
            .filter(|(k, _)| k.starts_with('_'))
            .map(|(k, _)| k.clone())
            .collect::<Vec<_>>();

        for k in prop_keys {
            self.properties.remove(&k);
        }
    }

    fn get_soft_cursor(&self) -> String {
        if self.get_option("soft_cursor") {
            "‸".to_string() // U+2038 ‸ CARET
        } else {
            String::new()
        }
    }

    fn delete_candidate(&mut self, cand: Option<Arc<dyn Candidate>>) -> bool {
        if let Some(cand) = cand {
            info!("Deleting candidate: {}", cand.text());
            {
                // self.delete_notifier.emit(self.clone());
                todo!()
            }
            true // CAVEAT: this doesn't mean anything is deleted for sure
        } else {
            false
        }
    }
}
