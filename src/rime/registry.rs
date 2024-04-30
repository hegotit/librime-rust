use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use log::{info, warn};
use once_cell::sync::Lazy;

pub trait ComponentBase: Send + Sync {}

pub struct Registry {
    map: Mutex<HashMap<String, Arc<dyn ComponentBase>>>,
}

impl Registry {
    pub fn new() -> Self {
        Self {
            map: Mutex::new(HashMap::new()),
        }
    }

    pub fn find(&self, name: &str) -> Option<Arc<dyn ComponentBase>> {
        let map = self.map.lock().unwrap();
        map.get(name).cloned()
    }

    pub fn register(&self, name: String, component: Arc<dyn ComponentBase>) {
        info!("registering component: {}", name);
        let mut map = self.map.lock().unwrap();
        if let Some(existing) = map.insert(name.clone(), component.clone()) {
            warn!("replacing previously registered component: {}", name);
            drop(existing);
        }
    }

    pub fn unregister(&self, name: &str) {
        info!("unregistering component: {}", name);
        let mut map = self.map.lock().unwrap();
        if map.remove(name).is_some() {
            info!("unregistered component: {}", name);
        }
    }

    pub fn clear(&self) {
        let mut map = self.map.lock().unwrap();
        map.clear();
    }

    pub fn instance() -> Arc<Self> {
        static INSTANCE: Lazy<Arc<Self>> = Lazy::new(|| {
            Arc::new(Self::new())
        });
        INSTANCE.clone()
    }
}
