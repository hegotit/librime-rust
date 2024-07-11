use crate::rime::common::PathExt;
use crate::rime::resource::{ResourceResolver, ResourceType};
use crate::rime::service::Service;
use crate::rime::RIME_VERSION;

use log::{error, info};

use std::collections::btree_map::{Iter, Range};
use std::fs;
use std::sync::{Arc, LazyLock};

pub enum IterFromPrefix<'b> {
    Iter(Iter<'b, String, String>),
    RangeWithPrefix(Range<'b, String, String>, &'b str),
}

impl<'b> Iterator for IterFromPrefix<'b> {
    type Item = (&'b String, &'b String);

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            IterFromPrefix::Iter(iter) => iter.next(),
            IterFromPrefix::RangeWithPrefix(range, prefix) => {
                range.filter(|(key, _)| key.starts_with(*prefix)).next()
            }
        }
    }
}

pub trait Db: DbAccessor + std::fmt::Debug {
    fn build(file_path: PathExt, db_name: &str) -> Self;

    fn remove(&self) -> bool;

    fn open(&mut self, enhanced: bool) -> bool;

    fn open_read_only(&mut self) -> bool;

    fn close(&mut self) -> bool;

    fn backup(&mut self, snapshot_file: &PathExt) -> bool;

    fn restore(&mut self, snapshot_file: &PathExt) -> bool;

    fn create_metadata(&mut self) -> bool;

    fn create_metadata_enhanced(&mut self, enhanced: bool) -> bool;

    fn meta_fetch(&self, key: &str) -> Option<String>;

    fn meta_update(&mut self, key: String, value: String) -> bool;

    fn fetch(&self, key: &str) -> Option<String>;

    fn update(&mut self, key: &str, value: &str) -> bool;

    fn erase(&mut self, key: &str) -> bool;

    fn name(&self) -> &str;

    fn loaded(&self) -> bool;
}

pub trait DbAccessor {
    fn reset(&self) -> bool;

    fn jump(&self, key: &str) -> bool;

    fn get_record_iter(&mut self, query_meta: bool, key: Option<&str>) -> IterFromPrefix;

    fn query(&mut self, key: Option<&str>) -> Option<IterFromPrefix>;
}

#[derive(Debug)]
pub struct BaseDb {
    pub(crate) name: String,
    file_path: PathExt,
    pub(crate) loaded: bool,
    pub(crate) readonly: bool,
    disabled: bool,
    pub(crate) prefix: String,
}

impl BaseDb {
    pub(crate) fn new(file_path: PathExt, name: &str) -> Self {
        Self {
            name: name.to_string(),
            file_path,
            loaded: false,
            readonly: false,
            disabled: false,
            prefix: String::new(),
        }
    }

    pub fn remove(&self) -> bool {
        if self.loaded {
            error!("Attempt to remove opened db '{}'.", self.name);
            return false;
        }
        fs::remove_file(&self.file_path).is_ok()
    }

    pub fn exists(&self) -> bool {
        self.file_path.exists()
    }

    pub(crate) fn name(&self) -> &str {
        &self.name
    }

    pub(crate) fn file_path(&self) -> &PathExt {
        &self.file_path
    }

    pub(crate) fn readonly(&self) -> bool {
        self.readonly
    }
    fn disabled(&self) -> bool {
        self.disabled
    }

    fn disable(&mut self) {
        self.disabled = true;
    }

    fn enable(&mut self) {
        self.disabled = false;
    }

    pub(crate) fn matches_prefix(&self, key: &str) -> bool {
        key.starts_with(&self.prefix)
    }
}

trait Transactional {
    fn begin_transaction(&self) -> bool {
        false
    }

    fn abort_transaction(&self) -> bool {
        false
    }

    fn commit_transaction(&self) -> bool {
        false
    }

    fn in_transaction(&self) -> bool;
}

trait Recoverable {
    fn recover(&self) -> bool;
}

struct DbComponentBase {
    db_resource_resolver: Box<ResourceResolver>,
}

static DB_RESOURCE_TYPE: LazyLock<ResourceType> = LazyLock::new(|| ResourceType {
    name: String::from("db"),
    prefix: String::new(),
    suffix: String::new(),
});

impl DbComponentBase {
    fn new() -> Self {
        Self {
            db_resource_resolver: Service::instance().create_resource_resolver(&DB_RESOURCE_TYPE),
        }
    }

    fn db_file_path(&self, name: &str, extension: &str) -> PathExt {
        let resource_id = format!("{}{}", name, extension);
        self.db_resource_resolver.resolve_path(&resource_id)
    }
}

// struct DbComponent /*<DbClass>*/ {
//    // db_class: PhantomData<DbClass>,
//    base: DbComponentBase,
//}

// impl<DbClass> DbComponent<DbClass> {
//      fn new() -> Self {
//         DbComponent {
//             db_class: PhantomData,
//             base: DbComponentBase::new(),
//         }
//     }
//
//      fn extension(&self) -> String {
//         todo!()
//     }
//
//      fn create(&self, name: &str) -> DbClass {
//         todo!()
//     }
// }

//impl Component for DbComponent {
//    fn create(&self, name: &str) {
//        todo!()
//    }
//}

// Additional implementations for ResourceResolver and other necessary components
//impl ResourceResolver {
//     fn resolve_path(&self, name: &str, extension: &str) -> PathBuf {
//        let mut path = PathBuf::from(name);
//        path.set_extension(extension);
//        path
//    }
//}
