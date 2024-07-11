use crate::rime::deployer::Deployer;
use crate::rime::resource::ResourceResolver;
use crate::rime::resource::ResourceType;
use std::sync::{LazyLock, Mutex, MutexGuard};

type SessionId = u64;

struct Session;

struct NotificationHandler;

pub(crate) struct Service {
    sessions: Mutex<Vec<(SessionId, Session)>>,
    deployer: Mutex<Deployer>,
    notification_handler: Mutex<Option<NotificationHandler>>,
    started: Mutex<bool>,
}

impl Service {
    fn new() -> Self {
        Service {
            sessions: Mutex::new(Vec::new()),
            deployer: Mutex::new(Deployer::new()),
            notification_handler: Mutex::new(None),
            started: Mutex::new(false),
        }
    }

    fn start_service(&self) {
        let mut started = self.started.lock().unwrap();
        *started = true;
    }

    fn stop_service(&self) {
        let mut started = self.started.lock().unwrap();
        *started = false;
    }

    fn create_session(&self) -> SessionId {
        let mut sessions = self.sessions.lock().unwrap();
        let session_id = 1; // 示例ID，实际应生成唯一ID
        sessions.push((session_id, Session));
        session_id
    }

    fn get_session(&self, session_id: SessionId) -> Option<MutexGuard<Vec<(SessionId, Session)>>> {
        let sessions = self.sessions.lock().unwrap();
        if sessions.iter().any(|(id, _)| *id == session_id) {
            Some(sessions)
        } else {
            None
        }
    }

    fn destroy_session(&self, session_id: SessionId) -> bool {
        let mut sessions = self.sessions.lock().unwrap();
        if let Some(index) = sessions.iter().position(|(id, _)| *id == session_id) {
            sessions.remove(index);
            true
        } else {
            false
        }
    }

    fn cleanup_stale_sessions(&self) {
        // 清理逻辑
    }

    fn cleanup_all_sessions(&self) {
        let mut sessions = self.sessions.lock().unwrap();
        sessions.clear();
    }

    fn set_notification_handler(&self, handler: NotificationHandler) {
        let mut notification_handler = self.notification_handler.lock().unwrap();
        *notification_handler = Some(handler);
    }

    fn clear_notification_handler(&self) {
        let mut notification_handler = self.notification_handler.lock().unwrap();
        *notification_handler = None;
    }

    fn notify(&self, session_id: SessionId, message_type: &str, message_value: &str) {
        // 通知逻辑
    }

    pub(crate) fn create_resource_resolver(&self, type_: &ResourceType) -> Box<ResourceResolver> {
        //let deployer = self.deployer();
        //let mut resolver = ResourceResolver::new(
        //    deployer.user_data_dir.clone(),
        //    deployer.shared_data_dir.clone(),
        //);
        //resolver.set_root_path(deployer.user_data_dir.clone());
        //resolver.set_fallback_root_path(deployer.shared_data_dir.clone());
        //Arc::new(Mutex::new(resolver))
        todo!()
    }

    fn create_user_specific_resource_resolver(&self, _type: &ResourceType) -> ResourceResolver {
        // 创建用户特定资源解析器逻辑
        todo!()
    }

    fn create_deployed_resource_resolver(&self, _type: &ResourceType) -> ResourceResolver {
        // 创建部署资源解析器逻辑
        todo!()
    }

    fn create_staging_resource_resolver(&self, _type: &ResourceType) -> ResourceResolver {
        // 创建预发布资源解析器逻辑
        todo!()
    }

    pub(crate) fn deployer(&self) -> MutexGuard<Deployer> {
        self.deployer.lock().unwrap()
    }

    fn disabled(&self) -> bool {
        let started = self.started.lock().unwrap();
        let deployer = self.deployer.lock().unwrap();
        !*started || deployer.is_maintenance_mode()
    }

    pub(crate) fn instance() -> &'static Self {
        static INSTANCE: LazyLock<Service> = LazyLock::new(|| Service::new());
        &INSTANCE
    }
}

fn main() {
    let service = Service::instance();
    service.start_service();
    // 使用service对象进行操作
}
