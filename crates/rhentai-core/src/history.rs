use crate::error::Result;
use crate::model::{HistoryEntry, ReadProgress};
use crate::store::{Store, current_time_secs};

#[derive(Clone)]
pub struct History {
    store: Store,
}

impl History {
    pub fn new(store: Store) -> Self {
        Self { store }
    }

    pub fn record_progress(&self, req: ReadProgress) -> Result<()> {
        let entry = HistoryEntry {
            target: req.target,
            title: req.title,
            thumb: req.thumb,
            category: req.category,
            page_count: req.page_count,
            last_page: req.page,
            read_at: current_time_secs(),
        };
        self.store.record_history(&entry)
    }

    pub fn list(&self) -> Result<Vec<HistoryEntry>> {
        self.store.list_history()
    }

    pub fn clear(&self) -> Result<()> {
        self.store.clear_history()
    }
}
