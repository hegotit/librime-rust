use log::{info, warn};
use std::collections::BTreeMap;
use std::sync::{Arc, LazyLock, Mutex};

use crate::rime::component::ComponentBase;

struct Registry {
    map: Mutex<BTreeMap<String, Box<dyn ComponentBase>>>,
}

impl Registry {
    fn new() -> Self {
        Self {
            map: Mutex::new(BTreeMap::new()),
        }
    }

    fn find(&self, name: &str) -> Option<Box<dyn ComponentBase>> {
        self.map.lock().unwrap().get(name).cloned()
    }

    fn register(&self, name: String, component: Box<dyn ComponentBase>) {
        info!("Registering component: {}", name);
        let mut map = self.map.lock().unwrap();
        if map.insert(name.clone(), component).is_some() {
            warn!("Replacing previously registered component: {}", name);
        }
    }

    fn unregister(&self, name: &str) {
        info!("Unregistering component: {}", name);
        self.map.lock().unwrap().remove(name);
    }

    fn clear(&self) {
        self.map.lock().unwrap().clear()
    }

    fn instance() -> Arc<Self> {
        static INSTANCE: LazyLock<Arc<Registry>> = LazyLock::new(|| Arc::new(Registry::new()));
        INSTANCE.clone()
    }
}
