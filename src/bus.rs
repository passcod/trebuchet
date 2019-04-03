use crossbeam_channel::{unbounded, Receiver, Sender, TrySendError};
use log::{debug, trace};
use std::collections::HashMap;
use std::fmt::Debug;
use std::thread::{Builder, JoinHandle};
use uuid::Uuid;

pub fn central<T: 'static + Clone + Debug + Send>() -> (Bus<T>, JoinHandle<()>) {
    let (central_tx, central_rx) = unbounded(); // enveloped

    let id = Uuid::default();
    let (bus_tx, bus_rx) = unbounded(); // bare
    let bus = Bus {
        id: id.clone(),
        to_central: central_tx,
        rx: bus_rx,
    };

    (
        bus,
        Builder::new()
            .name("bus central".into())
            .spawn(move || {
                let mut switch: HashMap<Uuid, Sender<(Uuid, T)>> = HashMap::new();
                switch.insert(id, bus_tx);

                for envelope in central_rx.iter() {
                    trace!("message on the bus: {:?}", envelope);
                    let mut dead = Vec::new();
                    match envelope {
                        Envelope::Exit => break,
                        Envelope::Broadcast { source, content } => {
                            for (id, tx) in &switch {
                                if let Err(TrySendError::Disconnected(_)) =
                                    tx.try_send((source, content.clone()))
                                {
                                    dead.push(id.clone());
                                }
                            }
                        }
                        Envelope::Direct {
                            source,
                            target,
                            content,
                        } => {
                            if let Some(tx) = switch.get(&target) {
                                if let Err(TrySendError::Disconnected(_)) =
                                    tx.try_send((source, content))
                                {
                                    dead.push(target.clone());
                                }
                            }
                        }
                        Envelope::Launch { id, tx } => {
                            switch.insert(id, tx);
                        }
                    }

                    if !dead.is_empty() {
                        trace!("burying dead buses: {:?}", dead);
                    }
                    for id in &dead {
                        switch.remove(id);
                    }
                }
            })
            .expect("failed to start bus central"),
    )
}

#[derive(Clone, Debug)]
pub struct Bus<T> {
    /// Unique bus instance id (zero for top level)
    pub id: Uuid,

    to_central: Sender<Envelope<T>>,
    rx: Receiver<(Uuid, T)>,
}

#[derive(Clone, Debug)]
pub enum Envelope<T> {
    Exit,
    Broadcast {
        source: Uuid,
        content: T,
    },
    Direct {
        source: Uuid,
        target: Uuid,
        content: T,
    },
    Launch {
        id: Uuid,
        tx: Sender<(Uuid, T)>,
    },
}

impl<T: 'static + Clone + Debug + Send> Bus<T> {
    pub fn launch(mut self) -> Self {
        let id = Uuid::new_v4();
        debug!("new bus: {}", id);

        let (tx, rx) = unbounded();
        self.send(Envelope::Launch { id: id.clone(), tx });

        self.id = id;
        self.rx = rx;
        self
    }

    /// Shut down the entire bus
    pub fn kill(self) {
        debug!("killing the bus");
        self.send(Envelope::Exit);
    }

    fn send(&self, envelope: Envelope<T>) {
        trace!("sending to central: {:?}", envelope);
        if let Err(err) = self.to_central.try_send(envelope) {
            debug!(
                "error on sending message to bus, likely shutting down? {:?}",
                err
            );
        }
    }

    pub fn send_to(&self, target: &Uuid, msg: T) {
        self.send(Envelope::Direct {
            source: self.id.clone(),
            target: target.clone(),
            content: msg,
        })
    }

    pub fn send_top(&self, msg: T) {
        self.send_to(&Uuid::default(), msg)
    }

    pub fn send_own(&self, msg: T) {
        self.send_to(&self.id, msg)
    }

    pub fn broadcast(&self, msg: T) {
        self.send(Envelope::Broadcast {
            source: self.id.clone(),
            content: msg,
        })
    }

    pub fn iter(&self) -> impl Iterator<Item = T> + '_ {
        self.rx.iter().map(|(_, c)| c)
    }

    pub fn iter_with_source(&self) -> impl Iterator<Item = (Uuid, T)> + '_ {
        self.rx.iter()
    }
}
