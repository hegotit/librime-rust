use std::ops::{Deref, DerefMut};

use crate::rime::dict::db::{Db, IterFromPrefix};

pub(crate) trait Sink {
    fn meta_put(&mut self, key: &str, value: &str) -> bool;

    fn put(&mut self, key: &str, value: &str) -> bool;

    fn read_from_source<'a, S>(&'a mut self, source: &'a mut S) -> i32
    where
        Self: Sized,
        S: Source,
    {
        source.dump(Some(self))
    }
}

pub(crate) trait Source {
    fn meta_get(&mut self) -> Option<IterFromPrefix>;

    fn get(&mut self) -> Option<IterFromPrefix>;

    fn get_by_key(&mut self, key: Option<&str>) -> Option<IterFromPrefix>;

    fn write_to_sink<'a, S>(&'a mut self, sink: &'a mut S) -> i32
    where
        S: Sink,
    {
        self.dump(Some(sink))
    }

    fn dump<'a, S>(&'a mut self, sink: Option<&'a mut S>) -> i32
    where
        S: Sink + ?Sized,
    {
        let Some(sink) = sink else {
            return 0;
        };

        let mut num_entries = 0;

        if let Some(iter) = self.meta_get() {
            for (key, value) in iter {
                if sink.meta_put(&key, &value) {
                    num_entries += 1;
                }
            }
        }

        if let Some(iter) = self.get() {
            for (key, value) in iter {
                if sink.put(&key, &value) {
                    num_entries += 1;
                }
            }
        }

        num_entries
    }
}

pub(crate) struct DbSink<'a, T: Db> {
    db: Option<&'a mut T>,
}

impl<'a, T: Db> DbSink<'a, T> {
    pub(crate) fn new(db: &'a mut T) -> Self {
        Self { db: Some(db) }
    }
}

impl<'a, T> Sink for DbSink<'a, T>
where
    T: Db,
{
    fn meta_put(&mut self, key: &str, value: &str) -> bool {
        if let Some(ref mut db) = self.db {
            db.meta_update(key.to_owned(), value.to_owned())
        } else {
            false
        }
    }

    fn put(&mut self, key: &str, value: &str) -> bool {
        if let Some(ref mut db) = self.db {
            db.update(key, value)
        } else {
            false
        }
    }
}

pub(crate) struct DbSource<'a, T: Db> {
    db: &'a mut T,
}

impl<'a, T: Db> Deref for DbSource<'a, T> {
    type Target = &'a mut T;

    fn deref(&self) -> &Self::Target {
        &self.db
    }
}

impl<'a, T: Db> DerefMut for DbSource<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.db
    }
}

impl<'a, T: Db> DbSource<'a, T> {
    pub(crate) fn new(db: &'a mut T) -> Self {
        Self { db }
    }
}

impl<'a, T> Source for DbSource<'a, T>
where
    T: Db,
{
    fn meta_get(&mut self) -> Option<IterFromPrefix> {
        if !self.loaded() {
            return None;
        }

        Some(self.get_record_iter(true, None))
    }

    fn get(&mut self) -> Option<IterFromPrefix> {
        self.get_by_key(None)
    }

    fn get_by_key(&mut self, key: Option<&str>) -> Option<IterFromPrefix> {
        if !self.loaded() {
            return None;
        }

        Some(self.get_record_iter(false, key))
    }
}
