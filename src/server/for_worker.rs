use crate::proto::Worker;
use std::sync::{Arc, RwLock};

pub trait WorkerSource {
    fn register_worker(&mut self, worker: Worker);
}

pub struct WorkerServer<W: WorkerSource> {
    source: RwLock<Arc<W>>,
}

impl<W: WorkerSource> ws::Handler for WorkerServer<W> {
    fn on_request(&mut self, req: &ws::Request) -> ws::Result<ws::Response> {
        if req.protocols()?.contains(&&"armstrong/worker") {
            let mut res = ws::Response::from_request(req)?;
            res.set_protocol("armstrong/worker");
            Ok(res)
        } else {
            Err(ws::Error::new(ws::ErrorKind::Protocol, "wrong protocol"))
        }
    }

    fn on_open(&mut self, _shake: ws::Handshake) -> ws::Result<()> {
        info!("connection accepted for worker");
        // send greeting
        Ok(())
    }

    fn on_message(&mut self, _msg: ws::Message) -> ws::Result<()> {
        // handle server messages
        Ok(())
    }

    fn on_shutdown(&mut self) {
        // info!() something out
    }
}
