use crate::client::Kind;
use crate::Bus;
use log::trace;

#[derive(Clone, Debug)]
pub enum Missive {
    Exit,
    Hello {
        app: String,
        kind: Kind,
        name: String,
        tags: Vec<String>,
    },
}

pub fn worker(_ws: ws::Sender, bus: Bus<Missive>) {
    for missive in bus.iter() {
        trace!("received bus message: {:?}", missive);
        match missive {
            Missive::Exit => {
                bus.send_top(Missive::Exit);
                break;
            }
            _ => {}
        }
    }
}
