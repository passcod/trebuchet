use arc_swap::{ArcSwap, ArcSwapOption};
use jsonrpc_core::{Id, Response};
use log::trace;
use rpds::HashTrieMap;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

#[derive(Default)]
pub struct Inflight {
    counter: AtomicUsize,
    store: ArcSwap<HashTrieMap<Id, ArcSwapOption<Sender<Response>>>>,
}

impl Inflight {
    pub fn launch(&self) -> (Id, Receiver<Response>) {
        trace!("incrementic atomic");
        let id = Id::Num(self.counter.fetch_add(1, Ordering::AcqRel) as u64);
        let (tx, rx) = channel();

        let atx = ArcSwapOption::new(Some(Arc::new(tx)));
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
            .map(|oatx| {
                trace!("moving and unwrapping arc over sender");
                let satx = oatx.swap(None).unwrap();
                Arc::try_unwrap(satx).unwrap()
            })
    }
}
