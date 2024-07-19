use crate::rime::candidate::{Candidate, CandidateList};
use crate::rime::filter::Filter;
use crate::rime::translation::{MergedTranslation, Translation};
use log::{error, info};
use std::sync::{Arc, PoisonError, RwLock, RwLockReadGuard};

#[derive(Default)]
struct Page {
    page_size: usize,
    page_no: usize,
    is_last_page: bool,
    candidates: CandidateList,
}

impl Page {
    fn new(
        page_size: usize,
        page_no: usize,
        is_last_page: bool,
        candidates: CandidateList,
    ) -> Self {
        Self {
            page_size,
            page_no,
            is_last_page,
            candidates,
        }
    }
}

pub(crate) struct Menu {
    merged: Arc<RwLock<MergedTranslation>>,
    result: Arc<RwLock<dyn Translation>>,
    candidates: CandidateList,
}

impl Menu {
    fn new() -> Self {
        let candidates = CandidateList::new();
        let merged_translation = MergedTranslation::new(candidates.clone());
        let merged = Arc::new(RwLock::new(merged_translation.clone()));
        let result = Arc::new(RwLock::new(merged_translation)) as Arc<RwLock<dyn Translation>>;
        Self {
            merged,
            result,
            candidates,
        }
    }

    fn add_translation(&mut self, translation: Arc<RwLock<dyn Translation>>) {
        if let Ok(mut merged) = self.merged.write() {
            merged.add_translation(Some(translation));
            info!("Updated total translations: {}", merged.size());
        } else {
            error!("Failed to acquire write lock");
        }
    }

    fn add_filter(&mut self, _filter: Box<dyn Filter>) {
        //let result = Arc::clone(&self.result);
        //filter.apply(result, &mut self.candidates);
        todo!()
    }

    pub(crate) fn prepare(&mut self, candidate_count: usize) -> usize {
        info!("Preparing {} candidates", candidate_count);

        // Pre-allocate space
        self.candidates
            .reserve(candidate_count.saturating_sub(self.candidates.len()));

        let mut continue_loop = true;
        while self.candidates.len() < candidate_count && continue_loop {
            match self.result.read() {
                Ok(result_read_guard) => {
                    if result_read_guard.exhausted() {
                        break;
                    }
                    drop(result_read_guard); // Release read lock, prepare to acquire write lock

                    match self.result.write() {
                        Ok(mut result_write_guard) => {
                            let cand = result_write_guard.next();
                            self.candidates.push(cand);
                            drop(result_write_guard); // Immediately release write lock
                        }
                        Err(e) => {
                            error!("Failed to acquire write lock: {}", e);
                            continue_loop = false;
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to acquire read lock: {}", e);
                    continue_loop = false;
                }
            }
        }

        self.candidates.len()
    }

    fn create_page(&mut self, page_size: usize, page_no: usize) -> Option<Page> {
        let start_pos = page_size * page_no;
        let mut end_pos = start_pos + page_size;

        let mut exhausted = match is_exhausted(&self.result.read()) {
            Ok(exhausted) => exhausted,
            Err(_) => return None,
        };

        if end_pos > self.candidates.len() {
            if !exhausted {
                end_pos = self.prepare(end_pos);
                exhausted = match is_exhausted(&self.result.read()) {
                    Ok(exhausted) => exhausted,
                    Err(_) => return None,
                };
            } else {
                end_pos = self.candidates.len();
            }

            if start_pos >= end_pos {
                return None;
            }
            end_pos = usize::min(start_pos + page_size, end_pos);
        }

        let mut page = Page::default();
        page.page_size = page_size;
        page.page_no = page_no;
        page.is_last_page = exhausted && end_pos == self.candidates.len();
        page.candidates = self.candidates[start_pos..end_pos].to_vec();
        Some(page)
    }

    pub(crate) fn get_candidate_at(&mut self, index: usize) -> Option<Arc<dyn Candidate>> {
        if index >= self.candidates.len() && index >= self.prepare(index + 1) {
            return None;
        }
        self.candidates[index].clone()
    }

    // CAVEAT: returns the number of candidates currently obtained,
    // rather than the total number of available candidates.
    fn candidate_count(&self) -> usize {
        return self.candidates.len();
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.candidates.is_empty() && {
            match self.result.read() {
                Ok(result_read_guard) => result_read_guard.exhausted(),
                Err(e) => {
                    error!("Failed to acquire read lock: {}", e);
                    false
                }
            }
        }
    }
}

// Function to read the exhausted state
fn is_exhausted(
    result: &Result<
        RwLockReadGuard<dyn Translation>,
        PoisonError<RwLockReadGuard<dyn Translation>>,
    >,
) -> Result<bool, ()> {
    match result {
        Ok(result_read_guard) => Ok(result_read_guard.exhausted()),
        Err(e) => {
            error!("Failed to acquire read lock: {}", e);
            Err(())
        }
    }
}
