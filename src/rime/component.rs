use crate::rime::registry::Registry;
use std::sync::{Arc, Mutex};

pub trait ComponentBase {
    fn create(&self, arg: &str) -> Box<dyn ComponentBase>;
}

pub struct Class<T: ComponentBase> {
    _marker: std::marker::PhantomData<T>,
}

impl<T: ComponentBase> Class<T> {
    pub fn require(name: &str) -> Option<Arc<Mutex<dyn ComponentBase>>> {
        Registry::instance().find(name);
        todo!()
    }
}

pub struct Component<T: ComponentBase> {
    _marker: std::marker::PhantomData<T>,
}

// impl<T: ComponentBase> Component<T> {
//     pub fn create(arg: &str) -> Box<dyn ComponentBase> {
//         // Box::new(T::create(arg))
//     }
// }
