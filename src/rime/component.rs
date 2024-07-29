pub(crate) trait ComponentBase: Send {
    fn create(&self, name: &str);

    fn clone_box(&self) -> Box<dyn ComponentBase>;
}

impl Clone for Box<dyn ComponentBase> {
    fn clone(&self) -> Box<dyn ComponentBase> {
        self.clone_box()
    }
}

trait Component: ComponentBase {
    fn create(&self, name: &str);
}
