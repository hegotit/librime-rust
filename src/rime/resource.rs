use crate::rime::common::PathExt;
use std::fs;

pub(crate) struct ResourceType {
    pub(crate) name: String,
    pub(crate) prefix: String,
    pub(crate) suffix: String,
}

impl ResourceType {
    pub(crate) fn new(name: &str, prefix: &str, suffix: &str) -> Self {
        Self {
            name: name.to_string(),
            prefix: prefix.to_string(),
            suffix: suffix.to_string(),
        }
    }
}

pub(crate) struct ResourceResolver {
    pub(crate) root_path: PathExt,
    type_: ResourceType,
}

impl ResourceResolver {
    fn set_root_path(&mut self, root_path: PathExt) {
        self.root_path = root_path;
    }

    pub(crate) fn resolve_path(&self, resource_id: &str) -> PathExt {
        let resource_name = format!("{}{}{}", self.type_.prefix, resource_id, self.type_.suffix);
        let resource_path = self.root_path.clone() / resource_name;
        let path_buf = fs::canonicalize(&resource_path).unwrap_or(resource_path.0);
        PathExt::new(path_buf)
    }
}
