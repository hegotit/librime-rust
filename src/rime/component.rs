use std::any::Any;

use crate::rime::deployer::Deployer;

pub(crate) trait ComponentBase: Any + Send + Sync {
    fn as_any(&self) -> &dyn Any;
}

pub(crate) trait DeploymentTask: ComponentBase {
    fn run(&self, deployer: &Deployer) -> bool;
}

impl<T: 'static + DeploymentTask> ComponentBase for T {
    fn as_any(&self) -> &dyn Any {
        self
    }
}
