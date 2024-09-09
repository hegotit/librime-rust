use std::collections::BTreeMap;
use std::fmt::Debug;
use std::ops::Bound::{Included, Unbounded};
use std::ops::{Deref, DerefMut};

use log::{error, info};

use crate::rime::common::PathExt;
use crate::rime::dict::db::{BaseDb, Db, DbAccessor, IterFromPrefix};
use crate::rime::dict::db_utils::{DbSink, DbSource};
use crate::rime::dict::tsv::{TsvReader, TsvWriter};
use crate::rime::dict::user_db::{TextFormat, UserDbHelper, PLAIN_USERDB_FORMAT};
use crate::rime::RIME_VERSION;

pub(crate) type TextDbData = BTreeMap<String, String>;

#[derive(Debug)]
pub struct TextDb<'a> {
    db: BaseDb,
    db_type: String,
    format: &'a TextFormat,
    metadata: TextDbData,
    data: TextDbData,
    modified: bool,
}

impl<'a> Deref for TextDb<'a> {
    type Target = BaseDb;

    fn deref(&self) -> &Self::Target {
        &self.db
    }
}

impl<'a> DerefMut for TextDb<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.db
    }
}

impl<'a> TextDb<'a> {
    pub(crate) fn new(
        file_path: PathExt,
        db_name: &str,
        db_type: &str,
        format: &'a TextFormat,
    ) -> Self {
        Self {
            db: BaseDb::new(file_path, db_name),
            db_type: db_type.to_string(),
            format,
            metadata: TextDbData::new(),
            data: TextDbData::new(),
            modified: false,
        }
    }
}

impl<'a> Db for TextDb<'a> {
    fn build(file_path: PathExt, db_name: &str) -> Self {
        Self::new(file_path, db_name, "userdb", PLAIN_USERDB_FORMAT.deref())
    }

    fn remove(&self) -> bool {
        self.db.remove()
    }

    fn open(&mut self, enhanced: bool) -> bool {
        if self.loaded() {
            return false;
        }

        self.readonly = false;
        self.loaded = !self.exists() || self.load_from_file(self.file_path().clone());

        if !self.loaded {
            error!("Error opening db '{}'.", self.name());
            return false;
        }

        if self.meta_fetch("/db_name").is_none() && !self.create_metadata_enhanced(enhanced) {
            error!("Error creating metadata");
            self.close();
            return false;
        }

        self.modified = false;
        true
    }

    fn open_read_only(&mut self) -> bool {
        if self.loaded() {
            return false;
        }

        self.readonly = false;
        self.loaded = self.exists() && self.load_from_file(self.file_path().clone());

        if !self.loaded {
            error!("Error opening db '{}' read-only.", self.name());
            return false;
        }

        self.readonly = true;
        self.modified = false;
        true
    }

    fn close(&mut self) -> bool {
        if !self.loaded() {
            return false;
        }

        if self.modified && !self.save_to_file(self.file_path().clone()) {
            return false;
        }

        self.loaded = false;
        self.readonly = false;
        self.clear();
        self.modified = false;

        true
    }

    fn backup(&mut self, snapshot_file: &PathExt) -> bool {
        if !self.loaded() {
            return false;
        }

        info!("Backing up db '{}' to {}", self.name(), snapshot_file);

        if !self.save_to_file(snapshot_file.clone()) {
            error!(
                "Failed to create snapshot file '{}' for db '{}'.",
                snapshot_file,
                self.name()
            );
            return false;
        }

        true
    }

    fn restore(&mut self, snapshot_file: &PathExt) -> bool {
        if !self.loaded() || self.readonly {
            return false;
        }

        if !self.load_from_file(snapshot_file.clone()) {
            error!(
                "Failed to restore db '{}' from '{}'.",
                self.name(),
                snapshot_file
            );
            return false;
        }

        self.modified = false;
        true
    }

    fn create_metadata(&mut self) -> bool {
        self.create_metadata_enhanced(false)
    }

    fn create_metadata_enhanced(&mut self, enhanced: bool) -> bool {
        info!("Creating metadata for db '{}'.", self.name());
        self.meta_update("/db_name".to_owned(), self.name().to_string())
            && self.meta_update("/rime_version".to_owned(), RIME_VERSION.to_string())
            && self.meta_update("/db_type".to_owned(), self.db_type.clone())
            && (!enhanced || UserDbHelper::new(self).update_user_info())
    }

    fn meta_fetch(&self, key: &str) -> Option<String> {
        if !self.loaded() {
            return None;
        }

        self.metadata.get(key).cloned()
    }

    fn meta_update(&mut self, key: String, value: String) -> bool {
        if !self.loaded() || self.readonly {
            return false;
        }

        info!("Update db metadata: {} => {}", key, &value);
        self.metadata.insert(key, value);
        self.modified = true;
        true
    }

    fn fetch(&self, key: &str) -> Option<String> {
        if !self.loaded {
            return None;
        }

        self.data.get(key).cloned()
    }

    fn update(&mut self, key: &str, value: &str) -> bool {
        if !self.loaded() || self.readonly() {
            return false;
        }

        info!("Update db entry: {} => {}", key, value);
        self.data.insert(key.to_string(), value.to_string());
        self.modified = true;
        true
    }

    fn erase(&mut self, key: &str) -> bool {
        if !self.loaded() || self.readonly() {
            return false;
        }

        info!("Erase db entry: {}", key);
        if self.data.remove(key).is_some() {
            self.modified = true;
            true
        } else {
            false
        }
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn loaded(&self) -> bool {
        self.loaded
    }
}

impl<'a> DbAccessor for TextDb<'a> {
    fn reset(&self) -> bool {
        todo!()
    }

    fn jump(&self, _key: &str) -> bool {
        todo!()
    }

    fn get_record_iter(&mut self, query_meta: bool, key: Option<&str>) -> IterFromPrefix {
        self.iter_from_prefix(query_meta, key)
    }

    fn query(&mut self, key: Option<&str>) -> Option<IterFromPrefix> {
        if !self.loaded() {
            return None;
        }

        Some(self.get_record_iter(false, key))
    }
}

impl<'a> TextDb<'a> {
    fn iter_from_prefix(&mut self, query_meta: bool, key: Option<&str>) -> IterFromPrefix {
        if let Some(k) = key {
            self.prefix = k.to_string();
        } else if !self.prefix.is_empty() {
            self.prefix.clear();
        }

        let map = if query_meta {
            &self.metadata
        } else {
            &self.data
        };

        if self.prefix.is_empty() {
            IterFromPrefix::Iter(map.iter())
        } else {
            let range = map.range((Included(self.prefix.to_owned()), Unbounded));
            IterFromPrefix::RangeWithPrefix(range, &self.prefix)
        }
    }
}

impl<'a> TextDb<'a> {
    fn load_from_file(&mut self, file_path: PathExt) -> bool {
        self.clear();
        let reader = TsvReader::new(file_path, self.format.parser.clone());

        let mut sink = DbSink::new(self);
        match reader >> &mut sink {
            Ok(entries) => {
                info!("{} entries loaded.", entries);
                true
            }
            Err(e) => {
                error!("Error reading entries: {}", e);
                false
            }
        }
    }

    fn save_to_file(&mut self, file_path: PathExt) -> bool {
        let mut writer = TsvWriter::new(file_path, self.format.formatter.clone());
        writer.file_description = self.format.file_description.clone();

        let mut source = DbSource::new(self);
        match writer << &mut source {
            Ok(entries) => {
                info!("{} entries saved.", entries);
                true
            }
            Err(e) => {
                error!("Error writing entries: {}", e);
                false
            }
        }
    }

    fn clear(&mut self) {
        self.metadata.clear();
        self.data.clear();
    }
}

impl<'a> Drop for TextDb<'a> {
    fn drop(&mut self) {
        if self.loaded() {
            self.close();
        }
    }
}
