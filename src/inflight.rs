use arc_swap::ArcSwap;
use jsonrpc_core::{Id, Response};
use log::trace;
use rpds::HashTrieMap;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc, Mutex,
};

#[derive(Default)]
pub struct Inflight {
    counter: AtomicUsize,
    store: ArcSwap<HashTrieMap<Id, Arc<Mutex<Sender<Response>>>>>,
}

impl Inflight {
    pub fn launch(&self) -> (Id, Receiver<Response>) {
        trace!("incrementic atomic");
        let id = Id::Num(self.counter.fetch_add(1, Ordering::AcqRel) as u64);
        let (tx, rx) = channel();

        let atx = Arc::new(Mutex::new(tx));
        self.store.rcu(|inner| {
            let id = id.clone();
            trace!("rcu over inflight log to insert {:?}", id);
            inner.insert(id, atx.clone())
        });

        (id, rx)
    }

    pub fn recall(&self, id: &Id) -> Option<Sender<Response>> {
        self.store
            .rcu(|inner| {
                let id = id.clone();
                trace!("rcu over inflight log to remove {:?}", id);
                inner.remove(&id)
            })
            .get(&id)
            .map(|atx| {
                trace!("moving and unwrapping arc over sender");
                let mtx = Arc::try_unwrap(atx.clone()).unwrap();
                mtx.into_inner().unwrap()
            })
    }
}
