use crate::proto::Worker;
use crate::server::WorkerSource;
use crate::system::System;
use std::collections::HashMap;

pub struct Agent {
    system: System,

    // worker name -> definition
    workers: HashMap<String, Worker>,
    // workers comm with the agent through JSON RPC over Websocket.
    // the agent communicates with the daemon through the same.
    // i.e. agents run a server and a client,
    //      workers run a client,
    //      daemon runs a server.
    //
    // Why JSON RPC? Simple, lightweight, well-established, can be hand-written in a pinch
    // Why Websocket? Duplex, inspectable, trivial to secure, can be used from browsers as-is
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
