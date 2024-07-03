use std::io::Read;
use std::sync::{Arc, RwLock};

use log::{error, info};

use crate::rime::config::config_data::ConfigData;
use crate::rime::config::config_types::{ConfigItem, ConfigList, ConfigMap, ValueType};

pub(crate) struct Config {
    pub(crate) data: Arc<RwLock<ConfigData>>,
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

    pub(crate) fn contains(&self, key: &str) -> bool {
        match self.data.read() {
            Ok(data) => Self::has(data.root.as_any().downcast_ref::<ConfigMap>(), key),
            Err(_) => {
                error!("Failed to acquire read lock");
                false
            }
        }
    }

    pub(crate) fn contains_all(&self, keys: &[&str]) -> bool {
        let data = match self.data.read() {
            Ok(data) => data,
            Err(_) => {
                error!("Failed to acquire read lock");
                return false;
            }
        };

        let map = match data.root.as_any().downcast_ref::<ConfigMap>() {
            Some(map) => map,
            None => return false,
        };

        keys.iter().all(|&key| {
            map.get(key)
                .map_or(false, |item| item.type_() != ValueType::Null)
        })
    }

    pub(crate) fn contains_scalar(&self, key: &str) -> bool {
        match self.data.read() {
            Ok(data) => Self::has_scalar(data.root.as_any().downcast_ref::<ConfigMap>(), key),
            Err(_) => {
                error!("Failed to acquire read lock");
                false
            }
        }
    }

    pub(crate) fn contains_list(&self, key: &str) -> bool {
        match self.data.read() {
            Ok(data) => Self::has_list(data.root.as_any().downcast_ref::<ConfigMap>(), key),
            Err(_) => {
                error!("Failed to acquire read lock");
                false
            }
        }
    }

    pub(crate) fn get_string(&self, key: &str) -> String {
        self.data
            .read()
            .ok()
            .and_then(|data| {
                data.root
                    .as_any()
                    .downcast_ref::<ConfigMap>()?
                    .get_string(key)
            })
            .unwrap_or_default()
    }

    pub(crate) fn get_bool(&self, key: &str) -> bool {
        self.data
            .read()
            .ok()
            .and_then(|data| {
                data.root
                    .as_any()
                    .downcast_ref::<ConfigMap>()?
                    .get_bool(key)
            })
            .unwrap_or_default()
    }

    pub(crate) fn get_int(&self, key: &str) -> i32 {
        self.data
            .read()
            .ok()
            .and_then(|data| data.root.as_any().downcast_ref::<ConfigMap>()?.get_int(key))
            .unwrap_or_default()
    }

    pub(crate) fn get_double(&self, key: &str) -> f64 {
        self.data
            .read()
            .ok()
            .and_then(|data| {
                data.root
                    .as_any()
                    .downcast_ref::<ConfigMap>()?
                    .get_double(key)
            })
            .unwrap_or_default()
    }

    pub(crate) fn get_item(&self, key: &str) -> Option<Arc<dyn ConfigItem>> {
        self.data
            .read()
            .ok()
            .and_then(|data| data.root.as_any().downcast_ref::<ConfigMap>()?.get(key))
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
    fn meets_condition<F>(map: Option<&ConfigMap>, key: &str, condition: F) -> bool
    where
        F: Fn(&ValueType) -> bool,
    {
        map.and_then(|map| map.get(key))
            .map_or(false, |item| condition(&item.type_()))
    }

    fn is_type(map: Option<&ConfigMap>, key: &str, value_type: ValueType) -> bool {
        Self::meets_condition(map, key, |item_type| *item_type == value_type)
    }

    pub(crate) fn has(map: Option<&ConfigMap>, key: &str) -> bool {
        Self::meets_condition(map, key, |item_type| *item_type != ValueType::Null)
    }

    pub(crate) fn has_scalar(map: Option<&ConfigMap>, key: &str) -> bool {
        Self::is_type(map, key, ValueType::Scalar)
    }

    pub(crate) fn has_list(map: Option<&ConfigMap>, key: &str) -> bool {
        Self::is_type(map, key, ValueType::List)
    }
}
