use crossbeam_channel::{Receiver, Sender, SendError, TrySendError, unbounded};
use log::{debug, trace};
use std::collections::HashMap;
use std::thread::{JoinHandle, spawn};
use uuid::Uuid;

pub fn central<T: 'static + Clone + Send>() -> (Bus<T>, JoinHandle<()>) {
    let (central_tx, central_rx) = unbounded(); // enveloped

    let id = Uuid::default();
    let (bus_tx, bus_rx) = unbounded(); // bare
    let bus = Bus { id: id.clone(), to_central: central_tx, rx: bus_rx };

    (bus, spawn(move || {
        let mut switch: HashMap<Uuid, Sender<T>> = HashMap::new();
        switch.insert(id, bus_tx);

        for envelope in central_rx.iter() {
            let mut dead = Vec::new();
            match envelope {
                Envelope::Exit => break,
                Envelope::Broadcast(msg) => {
                    for (id, tx) in &switch {
                        if let Err(TrySendError::Disconnected(_)) = tx.try_send(msg.clone()) {
                            dead.push(id.clone());
                        }
                    }
                },
                Envelope::Direct { target, content } => {
                    if let Some(tx) = switch.get(&target) {
                        if let Err(TrySendError::Disconnected(_)) = tx.try_send(content) {
                            dead.push(target.clone());
                        }
                    }
                },
                Envelope::Launch { id, tx } => {
                    switch.insert(id, tx);
                },
            }

            trace!("burying dead buses: {:?}", dead);
            for id in &dead {
                switch.remove(id);
            }
        }
    }))
}

#[derive(Clone)]
pub struct Bus<T> {
    /// Unique bus instance id (zero for top level)
    pub id: Uuid,

    to_central: Sender<Envelope<T>>,
    rx: Receiver<T>,
}

#[derive(Clone)]
pub enum Envelope<T> {
    Exit,
    Broadcast(T),
    Direct { target: Uuid, content: T },
    Launch { id: Uuid, tx: Sender<T> },
}

impl<T: 'static + Clone + Send> Bus<T> {
    pub fn launch(mut self) -> Self {
        let id = Uuid::new_v4();
        debug!("new bus: {}", id);

        let (tx, rx) = unbounded();
        self.to_central.send(Envelope::Launch { id: id.clone(), tx }).ok();
        // If it errors (disconnected), everything is coming down soon anyway

        self.id = id;
        self.rx = rx;
        self
    }

    /// Shut down the entire bus
    pub fn kill(self) {
        debug!("killing the bus");
        self.to_central.send(Envelope::Exit).ok();
        // If it errors, the bus is already dead.
    }

    pub fn try_send_to(&self, target: &Uuid, msg: T) -> Result<(), TrySendError<Envelope<T>>> {
        self.to_central.try_send(Envelope::Direct { target: target.clone(), content: msg })
    }

    pub fn send_to(&self, target: &Uuid, msg: T) -> Result<(), SendError<Envelope<T>>> {
        self.to_central.send(Envelope::Direct { target: target.clone(), content: msg })
    }

    pub fn try_broadcast(&self, msg: T) -> Result<(), TrySendError<Envelope<T>>> {
        self.to_central.try_send(Envelope::Broadcast(msg))
    }

    pub fn broadcast(&self, msg: T) -> Result<(), SendError<Envelope<T>>> {
        self.to_central.send(Envelope::Broadcast(msg))
    }

    pub fn iter(&self) -> impl Iterator<Item = T> + '_ {
        self.rx.iter()
    }
}
