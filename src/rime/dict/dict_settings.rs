use crate::rime::config::config_component::Config;
use crate::rime::config::config_types::{ConfigList, ConfigMap, ConfigValue};
use log::error;
use std::io::{BufRead, Cursor};
use std::sync::Arc;

const DEFAULT_VOCABULARY: &str = "essay";

pub struct DictSettings {
    pub(crate) config: Config,
}

impl DictSettings {
    pub fn new() -> Self {
        Self {
            config: Config::new(),
        }
    }

    pub fn load_dict_header(&mut self, stream: &mut dyn BufRead) -> bool {
        let mut header = String::new();
        let mut line = String::new();
        while stream.read_line(&mut line).is_ok() {
            let line = line.trim_end().to_string();
            header.push_str(&line);
            header.push('\n');
            if line == "..." {
                // Yaml doc ending
                break;
            }
        }
        let mut header_stream = Cursor::new(header);
        if !self.config.load_from_stream(&mut header_stream) {
            return false;
        }

        if !self.config.contains_all(&["name", "version"]) {
            error!("Incomplete dict header");
            return false;
        }
        true
    }

    pub fn is_empty(&self) -> bool {
        !self.config.contains("name")
    }

    pub fn dict_name(&self) -> String {
        self.config.get_string("name")
    }

    pub fn dict_version(&self) -> String {
        self.config.get_string("version")
    }

    pub fn sort_order(&self) -> String {
        self.config.get_string("sort")
    }

    pub fn use_preset_vocabulary(&self) -> bool {
        self.config.get_bool("use_preset_vocabulary") || self.config.contains_scalar("vocabulary")
    }

    pub fn vocabulary(&self) -> String {
        let result = self.config.get_string("vocabulary");
        if result.is_empty() {
            DEFAULT_VOCABULARY.to_string()
        } else {
            result
        }
    }

    pub fn use_rule_based_encoder(&self) -> bool {
        self.config
            .data
            .read()
            .ok()
            .and_then(|data| {
                let item = data
                    .root
                    .as_any()
                    .downcast_ref::<ConfigMap>()?
                    .get("encoder")?;

                let map = item.as_any().downcast_ref::<ConfigMap>();

                Some(Config::has_list(map, "rules"))
            })
            .unwrap_or_default()
    }

    pub fn max_phrase_length(&self) -> i32 {
        self.config.get_int("max_phrase_length")
    }

    pub fn min_phrase_weight(&self) -> f64 {
        self.config.get_double("min_phrase_weight")
    }

    pub fn get_tables(&self) -> Option<Arc<ConfigList>> {
        if self.is_empty() {
            return None;
        }
        let mut tables = ConfigList::new();
        tables.append(self.config.get_item("name"));

        self.config.data.read().ok().and_then(|data| {
            let item = data
                .root
                .as_any()
                .downcast_ref::<ConfigMap>()?
                .get("import_tables")?;

            let seq = &item.as_any().downcast_ref::<ConfigList>()?.seq;

            for config_item in seq {
                if let Some(config_value) = config_item
                    .as_ref()
                    .and_then(|item| item.as_any().downcast_ref::<ConfigValue>())
                {
                    let table = config_value.str();
                    if table == self.dict_name() {
                        error!("Cannot import '{}' from itself", table);
                    } else {
                        tables.append(config_item.clone());
                    }
                }
            }

            Some(())
        });

        Some(Arc::new(tables))
    }

    pub fn get_column_index(&self, column_label: &str) -> i32 {
        if !self.config.contains("columns") {
            return match column_label {
                "text" => 0,
                "code" => 1,
                "weight" => 2,
                _ => -1,
            };
        }

        self.config
            .data
            .read()
            .ok()
            .and_then(|data| {
                let item = data
                    .root
                    .as_any()
                    .downcast_ref::<ConfigMap>()?
                    .get("columns")?;

                let columns = item.as_any().downcast_ref::<ConfigList>()?;

                for (index, config_item) in columns.seq.iter().enumerate() {
                    if let Some(config_value) = config_item
                        .as_ref()
                        .and_then(|item| item.as_any().downcast_ref::<ConfigValue>())
                    {
                        if column_label == config_value.str() {
                            return Some(index as i32);
                        }
                    }
                }

                None
            })
            .unwrap_or(-1)
    }
}
