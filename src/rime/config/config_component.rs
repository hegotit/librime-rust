use crate::rime::config::config_data::ConfigData;
use crate::rime::config::config_types::{ConfigItem, ConfigList, ConfigMap, ValueType};
use log::{error, info};
use std::io::Read;
use std::sync::{Arc, RwLock};

pub(crate) struct Config {
    data: Arc<RwLock<ConfigData>>,
}

impl Config {
    pub(crate) fn new() -> Self {
        Self {
            data: Arc::new(RwLock::new(ConfigData::new())),
        }
    }

    pub(crate) fn load_from_stream(&mut self, stream: &mut dyn Read) -> bool {
        if let Ok(mut data) = self.data.write() {
            data.load_from_stream(stream)
        } else {
            error!("Failed to acquire write lock");
            false
        }
    }

    fn root_config_map(&self) -> Option<Arc<ConfigMap>> {
        self.data
            .read()
            .ok()
            .and_then(|data| data.root.as_any().downcast_ref::<Arc<ConfigMap>>().cloned())
            .or_else(|| {
                error!("Failed to acquire read lock");
                None
            })
    }

    pub(crate) fn contains(&self, key: &str) -> bool {
        self.root_config_map()
            .and_then(|map| Some(Self::has(Some(map), key)))
            .unwrap_or(false)
    }

    pub(crate) fn contains_all(&self, keys: &[&str]) -> bool {
        if let Some(map) = self.root_config_map() {
            keys.iter().all(|&key| {
                map.get(key)
                    .map_or(false, |item| item.type_() != ValueType::Null)
            })
        } else {
            false
        }
    }

    pub(crate) fn contains_scalar(&self, key: &str) -> bool {
        Self::has_scalar(self.root_config_map(), key)
    }

    pub(crate) fn contains_list(&self, key: &str) -> bool {
        Self::has_list(self.root_config_map(), key)
    }

    pub(crate) fn get_config_map(&self, key: &str) -> Option<Arc<ConfigMap>> {
        Self::get_nested_config_map(self.root_config_map(), key)
    }

    pub(crate) fn get_string(&self, key: &str) -> String {
        self.root_config_map()
            .and_then(|map| map.get_value(key))
            .map_or_else(String::new, |value| value.parse_string())
    }

    pub(crate) fn get_bool(&self, key: &str) -> bool {
        self.root_config_map()
            .and_then(|map| map.get_value(key))
            .map_or(false, |value| value.parse_bool())
    }

    pub(crate) fn get_int(&self, key: &str) -> i32 {
        self.root_config_map()
            .and_then(|map| map.get_value(key))
            .map_or(0, |value| value.parse_int().unwrap_or(0))
    }

    pub(crate) fn get_double(&self, key: &str) -> f64 {
        self.root_config_map()
            .and_then(|map| map.get_value(key))
            .map_or(0.0, |value| value.parse_double())
    }

    pub(crate) fn get_item(&self, key: &str) -> Option<Arc<dyn ConfigItem>> {
        self.root_config_map().and_then(|map| map.get(key))
    }

    pub(crate) fn get_list(&self, key: &str) -> Option<Arc<ConfigList>> {
        self.root_config_map()
            .and_then(|map| map.get(key))
            .and_then(|item| {
                item.as_any()
                    .downcast_ref::<Arc<ConfigList>>()
                    .map(Arc::clone)
            })
    }

    pub(crate) fn get_item_by_path(&self, path: &str) -> Option<Arc<dyn ConfigItem>> {
        info!("Read: {}", path);
        if let Ok(data) = self.data.read() {
            data.traverse(path)
        } else {
            error!("Failed to acquire write lock");
            None
        }
    }

    pub(crate) fn set_item(&self, item: Arc<dyn ConfigItem>) {
        if let Ok(mut data) = self.data.write() {
            data.root = item;
            self.set_modified()
        } else {
            error!("Failed to acquire write lock");
        }
    }

    pub(crate) fn set_modified(&self) {
        if let Ok(mut data) = self.data.write() {
            data.set_modified()
        } else {
            error!("Failed to acquire write lock");
        }
    }
}

impl Config {
    pub(crate) fn get_nested_config_map(
        base_map: Option<Arc<ConfigMap>>,
        key: &str,
    ) -> Option<Arc<ConfigMap>> {
        base_map
            .and_then(|map| {
                map.get(key).and_then(|item| {
                    item.as_any()
                        .downcast_ref::<Arc<ConfigMap>>()
                        .map(Arc::clone)
                })
            })
            .map_or(None, |value| Some(value))
    }

    fn meets_condition<F>(map: Option<Arc<ConfigMap>>, key: &str, condition: F) -> bool
    where
        F: Fn(&ValueType) -> bool,
    {
        map.and_then(|map| map.get(key))
            .map_or(false, |item| condition(&item.type_()))
    }

    fn is_type(map: Option<Arc<ConfigMap>>, key: &str, value_type: ValueType) -> bool {
        Self::meets_condition(map, key, |item_type| *item_type == value_type)
    }

    pub(crate) fn has(map: Option<Arc<ConfigMap>>, key: &str) -> bool {
        Self::meets_condition(map, key, |item_type| *item_type != ValueType::Null)
    }

    pub(crate) fn has_scalar(map: Option<Arc<ConfigMap>>, key: &str) -> bool {
        Self::is_type(map, key, ValueType::Scalar)
    }

    pub(crate) fn has_list(map: Option<Arc<ConfigMap>>, key: &str) -> bool {
        Self::is_type(map, key, ValueType::List)
    }
}
