use std::any::Any;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum ValueType {
    Null,
    Scalar,
    List,
    Map,
}

pub trait ConfigItem: std::fmt::Debug {
    fn type_(&self) -> ValueType;
    fn empty(&self) -> bool;
    fn as_any(&self) -> &dyn Any;
}

#[derive(Debug)]
struct BaseConfigItem {
    type_: ValueType,
}

impl BaseConfigItem {
    fn new(type_: ValueType) -> Self {
        BaseConfigItem { type_ }
    }
}

impl ConfigItem for BaseConfigItem {
    fn type_(&self) -> ValueType {
        self.type_
    }

    fn empty(&self) -> bool {
        matches!(self.type_, ValueType::Null)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Debug)]
pub struct ConfigValue {
    base: BaseConfigItem,
    value: String,
}

impl ConfigItem for ConfigValue {
    fn type_(&self) -> ValueType {
        self.base.type_
    }

    fn empty(&self) -> bool {
        self.value.is_empty()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl ConfigValue {
    fn new() -> Self {
        Self {
            base: BaseConfigItem::new(ValueType::Scalar),
            value: String::new(),
        }
    }

    fn from_bool(value: bool) -> Self {
        Self {
            base: BaseConfigItem::new(ValueType::Scalar),
            value: value.to_string(),
        }
    }

    fn from_int(value: i32) -> Self {
        Self {
            base: BaseConfigItem::new(ValueType::Scalar),
            value: value.to_string(),
        }
    }

    fn from_double(value: f64) -> Self {
        Self {
            base: BaseConfigItem::new(ValueType::Scalar),
            value: value.to_string(),
        }
    }

    pub fn from_str(value: &str) -> Self {
        Self {
            base: BaseConfigItem::new(ValueType::Scalar),
            value: value.to_string(),
        }
    }

    fn from_string(value: String) -> Self {
        Self {
            base: BaseConfigItem::new(ValueType::Scalar),
            value,
        }
    }

    pub(crate) fn parse_bool(&self) -> bool {
        if self.value.is_empty() {
            return false;
        }

        let bstr = self.value.to_lowercase();
        match bstr.as_str() {
            "true" => true,
            _ => false,
        }
    }

    pub(crate) fn parse_int(&self) -> Option<i32> {
        if self.value.is_empty() {
            return None;
        }

        // Try to parse hex number
        if self.value.starts_with("0x") {
            if let Ok(hex) = u32::from_str_radix(&self.value[2..], 16) {
                if let Ok(signed_hex) = i32::try_from(hex) {
                    return Some(signed_hex);
                }
            }
        }

        match self.value.parse::<i32>() {
            Ok(val) => Some(val),
            Err(_) => None,
        }
    }

    pub(crate) fn parse_double(&self) -> f64 {
        if self.value.is_empty() {
            return 0.0;
        }

        self.value.parse::<f64>().unwrap_or(0.0)
    }

    pub(crate) fn parse_string(&self) -> String {
        self.value.clone()
    }

    pub(crate) fn str(&self) -> &str {
        &self.value
    }
}

#[derive(Debug)]
pub struct ConfigList {
    base: BaseConfigItem,
    pub(crate) seq: Vec<Option<Arc<dyn ConfigItem>>>,
}

impl ConfigItem for ConfigList {
    fn type_(&self) -> ValueType {
        self.base.type_
    }

    fn empty(&self) -> bool {
        self.seq.is_empty()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl ConfigList {
    pub fn new() -> Self {
        Self {
            base: BaseConfigItem::new(ValueType::List),
            seq: Vec::new(),
        }
    }

    fn get_at(&self, i: usize) -> Option<Arc<dyn ConfigItem>> {
        self.seq.get(i).and_then(|item| item.clone())
    }

    pub(crate) fn get_value_at(&self, i: usize) -> Option<&ConfigValue> {
        self.seq.get(i).and_then(|item| {
            item.as_ref()
                .and_then(|item| item.as_any().downcast_ref::<ConfigValue>())
        })
    }

    pub(crate) fn get_str_at(&self, i: usize) -> Option<&str> {
        self.seq.get(i).and_then(|item| {
            item.as_ref().and_then(|item| {
                item.as_any()
                    .downcast_ref::<ConfigValue>()
                    .map(|cv| cv.str())
            })
        })
    }

    fn set_at(&mut self, i: usize, element: Arc<dyn ConfigItem>) {
        if i >= self.seq.len() {
            self.resize(i + 1);
        }
        self.seq[i] = Some(element);
    }

    pub(crate) fn insert(&mut self, i: usize, element: Option<Arc<dyn ConfigItem>>) {
        if i > self.seq.len() {
            self.resize(i);
        }
        self.seq.insert(i, element);
    }

    pub fn append(&mut self, element: Option<Arc<dyn ConfigItem>>) {
        self.seq.push(element);
    }

    fn resize(&mut self, size: usize) {
        self.seq.resize_with(size, || {
            Some(Arc::new(BaseConfigItem::new(ValueType::Null)))
        });
    }

    fn clear(&mut self) {
        self.seq.clear();
    }

    pub(crate) fn size(&self) -> usize {
        self.seq.len()
    }
}

#[derive(Debug)]
pub(crate) struct ConfigMap {
    base: BaseConfigItem,
    pub(crate) map: HashMap<String, Arc<dyn ConfigItem>>,
}

impl ConfigMap {
    pub(crate) fn new() -> Self {
        ConfigMap {
            base: BaseConfigItem::new(ValueType::Map),
            map: HashMap::new(),
        }
    }

    pub(crate) fn has_key(&self, key: &str) -> bool {
        self.map.contains_key(key)
    }

    pub(crate) fn get(&self, key: &str) -> Option<Arc<dyn ConfigItem>> {
        self.map.get(key).cloned()
    }

    fn get_value(&self, key: &str) -> Option<&ConfigValue> {
        self.map
            .get(key)
            .and_then(|item| item.as_any().downcast_ref::<ConfigValue>())
    }

    pub(crate) fn get_list(&self, key: &str) -> Option<&ConfigList> {
        self.map
            .get(key)
            .and_then(|item| item.as_any().downcast_ref::<ConfigList>())
    }

    pub(crate) fn get_str(&self, key: &str) -> Option<&str> {
        self.get_value(key).map(|value| value.str())
    }

    pub(crate) fn get_string(&self, key: &str) -> Option<String> {
        self.get_value(key).map(|value| value.parse_string())
    }

    pub(crate) fn get_bool(&self, key: &str) -> Option<bool> {
        self.get_value(key).map(|value| value.parse_bool())
    }

    pub(crate) fn get_int(&self, key: &str) -> Option<i32> {
        self.get_value(key).map(|value| value.parse_int())?
    }

    pub(crate) fn get_double(&self, key: &str) -> Option<f64> {
        self.get_value(key).map(|value| value.parse_double())
    }

    fn set(&mut self, key: String, element: Arc<dyn ConfigItem>) {
        self.map.insert(key, element);
    }

    fn clear(&mut self) {
        self.map.clear();
    }
}

impl ConfigItem for ConfigMap {
    fn type_(&self) -> ValueType {
        self.base.type_.clone()
    }

    fn empty(&self) -> bool {
        self.map.is_empty()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
