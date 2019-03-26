use crossbeam_channel::{bounded, Receiver, Sender};
use jsonrpc_core::{Id, Response};
use log::trace;
use std::collections::HashMap;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc, Mutex,
};

#[derive(Clone, Default)]
pub struct Inflight {
    counter: Arc<AtomicUsize>,

    // Mutex is appropriate here as we're always doing writes, and never just
    // reading. So there's no need for any structure that optimises for reads!
    // Perhaps a hashmap with more granular locking would help, though.
    store: Arc<Mutex<HashMap<Id, Sender<Response>>>>,
}

impl Inflight {
    pub fn launch(&self) -> (Id, Receiver<Response>) {
        trace!("incrementic atomic");
        let id = Id::Num(self.counter.fetch_add(1, Ordering::AcqRel) as u64);
        let (tx, rx) = bounded(0);

        {
            trace!("obtain inflight store lock");
            let mut map = self.store.lock().expect("inflight store poisoned");
            trace!("insert {:?} into inflight log", id);
            map.insert(id.clone(), tx);
            trace!("release inflight store lock");
        }

        (id, rx)
    }

    pub fn recall(&self, id: &Id) -> Option<Sender<Response>> {
        let otx = {
            trace!("obtain inflight store lock");
            let mut map = self.store.lock().expect("inflight store poisoned");
            trace!("remove {:?} from inflight log", id);
            map.remove(id).map(|tx| {
                trace!("{:?} was in log", id);
                tx
            })
        };
        trace!("release inflight store lock");
        otx
    }
}
