use log::{info, warn};
use std::collections::BTreeMap;
use std::sync::{Arc, LazyLock, Mutex};

use crate::rime::component::ComponentBase;

pub(crate) struct Registry {
    map: Mutex<BTreeMap<String, Arc<dyn ComponentBase>>>,
}

static INSTANCE: LazyLock<Registry> = LazyLock::new(|| Registry::new());

impl Registry {
    fn new() -> Self {
        Self {
            map: Mutex::new(BTreeMap::new()),
        }
    }

    pub(crate) fn find(&self, name: &str) -> Option<Arc<dyn ComponentBase>> {
        self.map.lock().unwrap().get(name).cloned()
    }

    fn register(&self, name: &str, component: Arc<dyn ComponentBase>) {
        info!("Registering component: {}", name);
        let mut map = self.map.lock().unwrap();
        if map.contains_key(name) {
            warn!("Replacing previously registered component: {}", name);
        }
        map.insert(name.to_string(), component);
        info!("Registered component: {}", name);
    }

    fn unregister(&self, name: &str) {
        info!("Unregistering component: {}", name);
        self.map.lock().unwrap().remove(name);
    }

    fn clear(&self) {
        self.map.lock().unwrap().clear()
    }

    fn instance() -> &'static Self {
        &INSTANCE
    }

    pub(crate) fn require(name: &str) -> Option<Arc<dyn ComponentBase>> {
        Registry::instance().find(name)
    }
}
