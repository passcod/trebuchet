use crate::inflight::Inflight;
use crate::message;
use futures::sync::oneshot::Receiver;
use jsonrpc_core::{Output, Params, Response};

pub use self::agent_to_core::*;

mod agent_to_core;

pub trait Client {
    fn sender(&self) -> &ws::Sender;
    fn inflight(&self) -> &Inflight;

    fn call(
        &self,
        method: String,
        params: Params,
        binary: Option<&[u8]>,
    ) -> ws::Result<Receiver<Response>> {
        let (id, rx) = self.inflight().launch();

        let msg: ws::Message = match binary {
            None => message::methodcall(method, params, id).into(),
            Some(raw) => message::add_binary(message::methodcall(method, params, id), raw).into(),
        };
        self.sender().send(msg)?;

        Ok(rx)
    }

    fn handle_response_out(&self, out: Output) -> ws::Result<()> {
        if let Some(tx) = self.inflight().recall(match out {
            Output::Failure(ref fail) => &fail.id,
            Output::Success(ref succ) => &succ.id,
        }) {
            tx.send(Response::Single(out)).expect("Internal comm error");
        }

        Ok(())
    }

    fn handle_response(&self, res: Response) -> ws::Result<()> {
        match res {
            Response::Batch(reses) => {
                for res in reses {
                    self.handle_response_out(res)?;
                }

                return Ok(());
            }
            Response::Single(out) => self.handle_response_out(out),
        }
    }

    fn notify(&self, method: String, params: Params, binary: Option<&[u8]>) -> ws::Result<()> {
        let msg: ws::Message = match binary {
            None => message::notification(method, params).into(),
            Some(raw) => message::add_binary(message::notification(method, params), raw).into(),
        };
        self.sender().send(msg)
    }
}
