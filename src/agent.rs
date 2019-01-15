use crate::proto::Worker;
use crate::server::WorkerSource;
use crate::system::System;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

#[derive(Default)]
pub struct Agent {
    system: System,

    // worker name -> definition
    workers: HashMap<String, Worker>,
}

impl Agent {
    pub fn arced() -> Arc<RwLock<Self>> {
        Arc::new(RwLock::new(Self::default()))
    }
}

impl WorkerSource for Agent {
    fn register_worker(&mut self, worker: Worker) {
        self.workers.insert(worker.name.clone(), worker);
    }

    fn unregister_worker(&mut self, name: &str) {
        self.workers.remove(name);
    }

    fn get_worker(&self, name: &str) -> Option<&Worker> {
        self.workers.get(name)
    }
}
