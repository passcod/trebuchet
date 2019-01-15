use crate::proto::Worker;
use crate::server::WorkerSource;
use crate::system::System;
use arc_swap::ArcSwap;
use rpds::HashTrieMap;
use std::sync::Arc;

#[derive(Clone, Default)]
pub struct AgentHandle(ArcSwap<Agent>);

#[derive(Clone, Default)]
pub struct Agent {
    system: Arc<System>,

    // worker name -> definition
    workers: HashTrieMap<String, Worker>,
}

impl Agent {
    pub fn update_workers(&self, workers: HashTrieMap<String, Worker>) -> Self {
        Self {
            system: self.system.clone(),
            workers,
        }
    }
}

impl WorkerSource for AgentHandle {
    fn register_worker(&self, worker: Worker) {
        self.0.rcu(|old| {
            let worker = worker.clone();
            old.update_workers(old.workers.insert(worker.name.clone(), worker))
        });
    }

    fn unregister_worker(&self, name: &str) {
        self.0
            .rcu(|old| old.update_workers(old.workers.remove(name)));
    }

    fn get_worker(&self, name: &str) -> Option<Worker> {
        (&self.0.lease().workers).clone().get(name).cloned()
    }
}
