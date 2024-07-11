use crate::rime::common::PathExt;
use crate::rime::resource::{ResourceResolver, ResourceType};
use crate::rime::service::Service;
use crate::rime::RIME_VERSION;
use log::{error, info};
use std::fs;
use std::sync::{Arc, LazyLock};

pub(crate) struct DbAccessor {
    prefix: String,
}

impl DbAccessor {
    pub(crate) fn new(prefix: &str) -> Self {
        Self {
            prefix: prefix.to_string(),
        }
    }

    pub(crate) fn reset(&self) -> bool {
        todo!()
    }

    pub(crate) fn jump(&self, key: &str) -> bool {
        todo!()
    }

    pub(crate) fn get_next_record(&self, key: &mut String, value: &mut String) -> bool {
        todo!()
    }

    pub(crate) fn exhausted(&self) -> bool {
        todo!()
    }

    fn matches_prefix(&self, key: &str) -> bool {
        key.starts_with(&self.prefix)
    }
}

pub(crate) struct Db {
    name: String,
    file_path: PathExt,
    loaded: bool,
    readonly: bool,
    disabled: bool,
}

impl Db {
    pub(crate) fn new(file_path: &PathExt, name: String) -> Self {
        Self {
            name,
            file_path: file_path.clone(),
            loaded: false,
            readonly: false,
            disabled: false,
        }
    }

    pub(crate) fn exists(&self) -> bool {
        self.file_path.exists()
    }

    pub(crate) fn remove(&self) -> bool {
        if self.loaded {
            error!("Attempt to remove opened db '{}'.", self.name);
            return false;
        }
        fs::remove_file(&self.file_path).is_ok()
    }

    pub(crate) fn open() -> bool {
        todo!()
    }

    pub(crate) fn open_read_only() -> bool {
        todo!()
    }

    pub(crate) fn close() -> bool {
        todo!()
    }

    pub(crate) fn backup(&self, snapshot_file: &PathExt) -> bool {
        todo!()
    }

    pub(crate) fn restore(&self, snapshot_file: &PathExt) -> bool {
        todo!()
    }

    pub(crate) fn create_metadata(&self) -> bool {
        info!("Creating metadata for db '{}'.", self.name);
        self.meta_update("/db_name", &self.name) && self.meta_update("/rime_version", RIME_VERSION)
    }

    pub(crate) fn meta_fetch(&self, key: &str, value: &mut String) -> bool {
        todo!()
    }

    pub(crate) fn meta_update(&self, key: &str, value: &str) -> bool {
        todo!()
    }

    pub(crate) fn query_metadata(&self) -> Arc<DbAccessor> {
        todo!()
    }

    pub(crate) fn query_all(&self) -> Arc<DbAccessor> {
        todo!()
    }

    pub(crate) fn query(&self, key: &str) -> Arc<DbAccessor> {
        todo!()
    }

    pub(crate) fn fetch(&self, key: &str, value: &mut String) -> bool {
        todo!()
    }

    pub(crate) fn update(&self, key: &str, value: &str) -> bool {
        todo!()
    }

    pub(crate) fn erase(&self, key: &str) -> bool {
        todo!()
    }

    pub(crate) fn name(&self) -> &str {
        &self.name
    }

    pub(crate) fn file_path(&self) -> &PathExt {
        &self.file_path
    }

    pub(crate) fn loaded(&self) -> bool {
        self.loaded
    }

    pub(crate) fn readonly(&self) -> bool {
        self.readonly
    }

    pub(crate) fn disabled(&self) -> bool {
        self.disabled
    }

    pub(crate) fn disable(&mut self) {
        self.disabled = true;
    }

    pub(crate) fn enable(&mut self) {
        self.disabled = false;
    }
}

pub(crate) trait Transactional {
    fn begin_transaction(&self) -> bool {
        false
    }

    fn abort_transaction(&self) -> bool {
        false
    }

    fn commit_transaction(&self) -> bool {
        false
    }

    fn in_transaction(&self) -> bool {
        false
    }
}

pub(crate) trait Recoverable {
    fn recover(&self) -> bool;
}

pub(crate) struct DbComponentBase {
    db_resource_resolver: Box<ResourceResolver>,
}

static DB_RESOURCE_TYPE: LazyLock<ResourceType> = LazyLock::new(|| ResourceType {
    name: String::from("db"),
    prefix: String::new(),
    suffix: String::new(),
});

impl DbComponentBase {
    pub(crate) fn new() -> Self {
        Self {
            db_resource_resolver: Service::instance().create_resource_resolver(&DB_RESOURCE_TYPE),
        }
    }

    pub(crate) fn db_file_path(&self, name: &str, extension: &str) -> PathExt {
        let resource_id = format!("{}{}", name, extension);
        self.db_resource_resolver.resolve_path(&resource_id)
    }
}

//pub(crate) struct DbComponent /*<DbClass>*/ {
//    // db_class: PhantomData<DbClass>,
//    base: DbComponentBase,
//}

// impl<DbClass> DbComponent<DbClass> {
//     pub(crate) fn new() -> Self {
//         DbComponent {
//             db_class: PhantomData,
//             base: DbComponentBase::new(),
//         }
//     }
//
//     pub(crate) fn extension(&self) -> String {
//         todo!()
//     }
//
//     pub(crate) fn create(&self, name: &str) -> DbClass {
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
//    pub(crate) fn resolve_path(&self, name: &str, extension: &str) -> PathBuf {
//        let mut path = PathBuf::from(name);
//        path.set_extension(extension);
//        path
//    }
//}
