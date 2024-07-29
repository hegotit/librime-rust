use std::any::Any;
use std::collections::VecDeque;
use std::future::Future;
use std::pin::Pin;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};
use std::thread;

use crate::rime::common::PathExt;

trait DeploymentTask: Any {
    fn run(&self, deployer: &mut Deployer) -> bool;
}

struct Deployer {
    shared_data_dir: PathExt,
    user_data_dir: PathExt,
    prebuilt_data_dir: PathExt,
    staging_dir: PathExt,
    sync_dir: PathExt,
    user_id: String,
    distribution_name: String,
    distribution_code_name: String,
    distribution_version: String,
    app_name: String,
    pending_tasks: Arc<Mutex<VecDeque<Box<dyn DeploymentTask + Send>>>>,
    maintenance_mode: Arc<Mutex<bool>>,
}

impl Deployer {
    fn new() -> Self {
        Self {
            shared_data_dir: PathExt::from("."),
            user_data_dir: PathExt::from("."),
            prebuilt_data_dir: PathExt::from("build"),
            staging_dir: PathExt::from("build"),
            sync_dir: PathExt::from("sync"),
            user_id: String::from("unknown"),
            distribution_name: String::new(),
            distribution_code_name: String::new(),
            distribution_version: String::new(),
            app_name: String::new(),
            pending_tasks: Arc::new(Mutex::new(VecDeque::new())),
            maintenance_mode: Arc::new(Mutex::new(false)),
        }
    }

    fn run_task(&mut self, task_name: &str, arg: Box<dyn Any>) -> bool {
        // Implement task retrieval and creation logic here
        // For demonstration, assume task retrieval always fails
        eprintln!("unknown deployment task: {}", task_name);
        false
    }

    fn schedule_task(&mut self, task: Box<dyn DeploymentTask + Send>) {
        let mut tasks = self.pending_tasks.lock().unwrap();
        tasks.push_back(task);
    }

    fn next_task(&self) -> Option<Box<dyn DeploymentTask + Send>> {
        let mut tasks = self.pending_tasks.lock().unwrap();
        tasks.pop_front()
    }

    fn has_pending_tasks(&self) -> bool {
        let tasks = self.pending_tasks.lock().unwrap();
        !tasks.is_empty()
    }

    fn run(&self) -> bool {
        println!("running deployment tasks:");
        let (tx, rx): (Sender<bool>, Receiver<bool>) = channel();

        let mut success = 0;
        let mut failure = 0;

        loop {
            //while let Some(task) = self.next_task() {
            //    let tx_clone = tx.clone();
            //    let mut task_clone = task;
            //    let deployer = self.clone();
            //    thread::spawn(move || {
            //        let result = task_clone.run(&mut deployer);
            //        tx_clone.send(result).unwrap();
            //    });
            //}

            match rx.recv() {
                Ok(result) => {
                    if result {
                        success += 1;
                    } else {
                        failure += 1;
                    }
                }
                Err(_) => break,
            }

            println!(
                "{} tasks ran: {} success, {} failure.",
                success + failure,
                success,
                failure
            );
            if !self.has_pending_tasks() {
                break;
            }
        }

        failure == 0
    }

    fn start_work(&self, maintenance_mode: bool) -> bool {
        {
            let mut mm = self.maintenance_mode.lock().unwrap();
            *mm = maintenance_mode;
        }

        if !self.has_pending_tasks() {
            return false;
        }

        println!("starting work thread for tasks.");
        let deployer = self.clone();
        thread::spawn(move || {
            deployer.run();
        });

        true
    }

    fn start_maintenance(&self) -> bool {
        self.start_work(true)
    }

    fn is_working(&self) -> bool {
        // Simplified checking mechanism for demo purposes
        self.has_pending_tasks()
    }

    fn is_maintenance_mode(&self) -> bool {
        let mm = self.maintenance_mode.lock().unwrap();
        *mm && self.is_working()
    }

    fn join_work_thread(&self) {
        // Simplified join logic for demo purposes
    }

    fn join_maintenance_thread(&self) {
        self.join_work_thread();
    }

    fn user_data_sync_dir(&self) -> PathExt {
        self.sync_dir.join(&self.user_id)
    }
}

impl Clone for Deployer {
    fn clone(&self) -> Self {
        Self {
            shared_data_dir: self.shared_data_dir.clone(),
            user_data_dir: self.user_data_dir.clone(),
            prebuilt_data_dir: self.prebuilt_data_dir.clone(),
            staging_dir: self.staging_dir.clone(),
            sync_dir: self.sync_dir.clone(),
            user_id: self.user_id.clone(),
            distribution_name: self.distribution_name.clone(),
            distribution_code_name: self.distribution_code_name.clone(),
            distribution_version: self.distribution_version.clone(),
            app_name: self.app_name.clone(),
            pending_tasks: Arc::clone(&self.pending_tasks),
            maintenance_mode: Arc::clone(&self.maintenance_mode),
        }
    }
}
