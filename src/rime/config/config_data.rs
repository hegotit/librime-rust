use std::fs::{self, File};
use std::io::Read;
use std::sync::{Arc, RwLock};

use hashlink::LinkedHashMap;
use log::{error, info, warn};
use yaml_rust2::{Yaml, YamlEmitter, YamlLoader};

use crate::rime::common::PathExt;
use crate::rime::config::config_types::{
    ConfigItem, ConfigList, ConfigMap, ConfigValue, ValueType,
};

type ConfigCompiler = (); // 这里假设ConfigCompiler的具体类型

#[derive(Debug)]
enum ListPos {
    After,
    Before,
    Last,
    Next,
}

impl ListPos {
    fn from_str(s: &str) -> Option<Self> {
        match s {
            s if s.starts_with("after") => Some(ListPos::After),
            s if s.starts_with("before") => Some(ListPos::Before),
            s if s.starts_with("last") => Some(ListPos::Last),
            s if s.starts_with("next") => Some(ListPos::Next),
            _ => None,
        }
    }

    fn len(&self) -> usize {
        match self {
            ListPos::After => "after".len(),
            ListPos::Before => "before".len(),
            ListPos::Last => "last".len(),
            ListPos::Next => "next".len(),
        }
    }
}

pub(crate) struct ConfigData {
    file_path: Option<PathExt>,
    modified: bool,
    auto_save: bool,
    pub(crate) root: Arc<dyn ConfigItem>,
}

impl Drop for ConfigData {
    fn drop(&mut self) {
        if self.auto_save {
            self.save();
        }
    }
}

impl ConfigData {
    pub(crate) fn new() -> Self {
        Self {
            file_path: None,
            modified: false,
            auto_save: false,
            root: Arc::new(ConfigMap::new()),
        }
    }

    // Returns whether actually saved to file.
    pub(crate) fn save(&mut self) -> bool {
        if self.modified {
            if let Some(file_path) = self.file_path.take() {
                let result = self.save_to_file(&file_path);
                self.file_path = Some(file_path);
                return result;
            }
        }
        false
    }

    pub(crate) fn save_to_file(&mut self, file_path: &PathExt) -> bool {
        self.modified = false;
        if file_path.as_os_str().is_empty() {
            // Not really saving
            return false;
        }
        info!("Saving config file '{}'.", file_path.display());

        if let Ok(mut out) = File::create(file_path) {
            self.save_to_stream(&mut out)
        } else {
            false
        }
    }

    pub(crate) fn save_to_stream(&self, stream: &mut dyn std::io::Write) -> bool {
        let Some(yaml) = self.convert_to_yaml(&self.root) else {
            return false;
        };

        let mut output = String::new();
        let mut emitter = YamlEmitter::new(&mut output);
        if let Err(e) = emitter.dump(&yaml[0]) {
            error!("Error emitting YAML: {}", e);
            return false;
        }

        if let Err(e) = stream.write_all(output.as_bytes()) {
            error!("Failed to save config to stream: {}", e);
            return false;
        }
        true
    }

    pub(crate) fn load_from_file(
        &mut self,
        file_path: &PathExt,
        _compiler: &ConfigCompiler,
    ) -> bool {
        // Update status
        self.file_path = Some(file_path.to_owned());
        self.modified = false;

        if !file_path.exists() {
            if !file_path.ends_with(".custom.yaml") {
                warn!("Nonexistent config file '{}'.", file_path.display());
            }
            self.reset_root();
            return false;
        }

        info!("Loading config file '{}'.", file_path.display());

        match self.read_and_parse_file(file_path) {
            Ok(root) => {
                self.root = root;
                true
            }
            Err(e) => {
                error!(
                    "Failed to load config file '{}': {}",
                    file_path.display(),
                    e
                );
                self.reset_root();
                false
            }
        }
    }

    fn read_and_parse_file(&self, file_path: &PathExt) -> Result<Arc<dyn ConfigItem>, String> {
        let contents = self.read_file(file_path).map_err(|e| e.to_string())?;
        self.parse_yaml(&contents)
    }

    fn read_file(&self, file_path: &PathExt) -> Result<String, std::io::Error> {
        fs::read_to_string(file_path)
    }

    fn parse_yaml(&self, contents: &str) -> Result<Arc<dyn ConfigItem>, String> {
        let docs = YamlLoader::load_from_str(contents).map_err(|e| e.to_string())?;
        let doc = docs.get(0).ok_or("YAML document is empty")?;
        self.convert_from_yaml(doc)
            .ok_or_else(|| "Error parsing YAML".to_string())
    }

    pub(crate) fn load_from_stream(&mut self, stream: &mut dyn Read) -> bool {
        let mut buffer = String::new();
        if stream.read_to_string(&mut buffer).is_err() {
            error!("Failed to load config from stream");
            return false;
        }

        match self.parse_yaml(&buffer) {
            Ok(root) => {
                self.root = root;
                true
            }
            Err(e) => {
                error!("Error parsing YAML from stream: {}", e);
                false
            }
        }
    }

    fn convert_from_yaml(&self, node: &Yaml) -> Option<Arc<dyn ConfigItem>> {
        let _ = node;
        todo!()
    }

    fn convert_to_yaml(&self, node: &Arc<dyn ConfigItem>) -> Option<Yaml> {
        match node.type_() {
            ValueType::Scalar => {
                if let Some(config) = node.as_any().downcast_ref::<ConfigValue>() {
                    Some(Yaml::String(config.str().to_string()))
                } else {
                    None
                }
            }
            ValueType::List => {
                let array: Vec<Yaml> =
                    if let Some(config) = node.as_any().downcast_ref::<Arc<RwLock<ConfigList>>>() {
                        config
                            .read()
                            .unwrap()
                            .seq
                            .iter()
                            .filter_map(|item| {
                                item.as_ref().and_then(|item| self.convert_to_yaml(item))
                            })
                            .collect()
                    } else {
                        Vec::new()
                    };

                if array.is_empty() {
                    return None;
                }
                Some(Yaml::Array(array))
            }
            ValueType::Map => {
                let mut map = LinkedHashMap::new();
                if let Some(config) = node.as_any().downcast_ref::<ConfigMap>() {
                    for (key, value) in &config.map {
                        if value.type_() != ValueType::Null {
                            if let Some(value) = self.convert_to_yaml(&value) {
                                map.insert(Yaml::String(key.to_string()), value);
                            }
                        }
                    }
                }
                if map.is_empty() {
                    return None;
                }
                Some(Yaml::Hash(map))
            }
            _ => None,
        }
    }

    pub(crate) fn is_list_item_reference(key: &str) -> bool {
        key.len() > 1
            && key.starts_with('@')
            && key.chars().nth(1).map_or(false, |c| c.is_alphanumeric())
    }

    pub(crate) fn format_list_index(index: usize) -> String {
        format!("@{}", index)
    }

    pub(crate) fn resolve_list_index(
        &self,
        item: Arc<dyn ConfigItem>,
        key: &str,
        read_only: bool,
    ) -> usize {
        if !Self::is_list_item_reference(key) {
            return 0;
        }

        let Some(list) = item.as_any().downcast_ref::<Arc<RwLock<ConfigList>>>() else {
            return 0;
        };

        let mut cursor = 1;
        let mut index = 0;
        let mut will_insert = false;

        if let Some(pos) = ListPos::from_str(&key[cursor..]) {
            cursor += pos.len();
            match pos {
                ListPos::Next => {
                    let list = list.read().unwrap();
                    index = list.seq.len()
                }
                ListPos::Before => will_insert = true,
                ListPos::After => {
                    index += 1; // After i == before i+1
                    will_insert = true;
                }
                _ => (),
            }
        }

        if cursor < key.len() && key[cursor..].starts_with(' ') {
            cursor += 1;
        }

        if let Some(ListPos::Last) = ListPos::from_str(&key[cursor..]) {
            //cursor += ListPosition::Last.len();
            let list = list.read().unwrap();
            index += list.seq.len();
            if index != 0 {
                // When list is empty, (before|after) last == 0
                index -= 1;
            }
        } else {
            if let Ok(parsed_index) = key[cursor..].parse::<usize>() {
                index += parsed_index;
            }
        }

        if will_insert && !read_only {
            let mut list = list.write().unwrap();
            list.insert(index, None); // Insert null equivalent in Yaml-rust
        }

        index
    }

    fn type_checked_copy_on_write(
        node: Option<Arc<dyn ConfigItem>>,
        key: &str,
    ) -> Option<Arc<dyn ConfigItem>> {
        // Special case to allow editing current node by __append: __merge: /+: /=:
        if key.is_empty() {
            return node;
        }

        let is_list = Self::is_list_item_reference(key);
        let expected_node_type = if is_list {
            ValueType::List
        } else {
            ValueType::Map
        };

        if let Some(ref node) = node {
            if node.type_() != expected_node_type {
                error!("Copy on write failed; incompatible node type: {}", key);
                return None;
            }
        }

        node
    }

    pub(crate) fn traverse_write(&self, path: &str, item: Arc<dyn ConfigItem>) -> bool {
        let _ = item;
        let _ = path;
        todo!()
    }

    pub(crate) fn traverse(&self, path: &str) -> Option<Arc<dyn ConfigItem>> {
        let _ = path;
        todo!()
    }

    pub(crate) fn split_path(path: &str) -> Vec<String> {
        path.split('/').map(|s| s.to_string()).collect()
    }

    pub(crate) fn join_path(keys: &[String]) -> String {
        keys.join("/")
    }

    pub(crate) fn file_path(&self) -> Option<&PathExt> {
        self.file_path.as_ref()
    }

    pub(crate) fn modified(&self) -> bool {
        self.modified
    }

    pub(crate) fn set_modified(&mut self) {
        self.modified = true;
    }

    pub(crate) fn set_auto_save(&mut self, auto_save: bool) {
        self.auto_save = auto_save;
    }

    fn reset_root(&mut self) {
        self.root = Arc::new(ConfigMap::new())
    }
}

#[test]
fn test() {
    let mut config_data = ConfigData::new();
    config_data.set_auto_save(true);
    config_data.set_modified();
    println!("Modified: {}", config_data.modified());
}
