use std::any::Any;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Copy, Clone, PartialEq)]
pub(crate) enum ValueType {
    Null,
    Scalar,
    List,
    Map,
}

pub(crate) trait ConfigItem {
    fn type_(&self) -> ValueType;
    fn empty(&self) -> bool;
    fn as_any(&self) -> &dyn Any;
}

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

pub(crate) struct ConfigValue {
    base: BaseConfigItem,
    value: String,
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

    fn from_str(value: &str) -> Self {
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

pub(crate) struct ConfigList {
    base: BaseConfigItem,
    pub(crate) seq: Vec<Option<Arc<dyn ConfigItem>>>,
}

impl ConfigList {
    pub(crate) fn new() -> Self {
        Self {
            base: BaseConfigItem::new(ValueType::List),
            seq: Vec::new(),
        }
    }

    fn get_at(&self, i: usize) -> Option<Arc<dyn ConfigItem>> {
        self.seq.get(i).and_then(|item| item.clone())
    }

    pub(crate) fn get_value_at(&self, i: usize) -> Option<Arc<ConfigValue>> {
        self.seq.get(i).and_then(|item| {
            item.as_ref().and_then(|item| {
                item.as_any()
                    .downcast_ref::<Arc<ConfigValue>>()
                    .map(Arc::clone)
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

    pub(crate) fn append(&mut self, element: Option<Arc<dyn ConfigItem>>) {
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

    pub(crate) fn get_value(&self, key: &str) -> Option<Arc<ConfigValue>> {
        self.map.get(key).and_then(|item| {
            item.as_any()
                .downcast_ref::<Arc<ConfigValue>>()
                .map(Arc::clone)
        })
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
