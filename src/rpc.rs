use crate::inflight::Inflight;
use crate::message;
use futures::Future;
use jsonrpc_core::{IoHandler, Output, Params, Response};
use std::sync::mpsc::Receiver;

pub trait RpcHandler {
    const PROTOCOL: &'static str;

    fn inflight(&self) -> &Inflight;
    fn rpc(&self) -> &IoHandler;
    fn sender(&self) -> &ws::Sender;

    fn call(
        &self,
        method: &str,
        params: Params,
        binary: Option<&[u8]>,
    ) -> ws::Result<Receiver<Response>> {
        let (id, rx) = self.inflight().launch();
        trace!("requested new inflight id: {:?}", id);

        let msg: ws::Message = match binary {
            None => message::methodcall(method.into(), params, id).into(),
            Some(raw) => {
                message::add_binary(message::methodcall(method.into(), params, id), raw).into()
            }
        };

        trace!("built method call (and about to send): {:?}", msg);
        self.sender().send(msg)?;

        Ok(rx)
    }

    fn notify(&self, method: &str, params: Params, binary: Option<&[u8]>) -> ws::Result<()> {
        let msg: ws::Message = match binary {
            None => message::notification(method.into(), params).into(),
            Some(raw) => {
                message::add_binary(message::notification(method.into(), params), raw).into()
            }
        };

        trace!("built notification (and about to send): {:?}", msg);
        self.sender().send(msg)
    }

    fn respawn(&self, chan: Receiver<Response>, cb: fn(Response)) {
        trace!("spawn thread for response");
        std::thread::spawn(move || {
            let res = chan.recv().expect("Internal comm error");
            trace!("got response from agent: {:?}", res);
            cb(res);
        });
    }

    fn rpc_build_request(&self, url: &url::Url) -> ws::Result<ws::Request> {
        let mut req = ws::Request::from_url(url)?;
        req.add_protocol(Self::PROTOCOL);
        trace!("built handshake request {:?}", req);
        Ok(req)
    }

    fn rpc_on_response(&self, res: &ws::Response) -> ws::Result<()> {
        trace!("got handshake response {:?}", res);
        if let Some(proto) = res.protocol()? {
            if proto == Self::PROTOCOL {
                return Ok(());
            }
        }

        Err(ws::Error::new(ws::ErrorKind::Protocol, "wrong protocol"))
    }

    fn rpc_on_request(&mut self, req: &ws::Request) -> ws::Result<ws::Response> {
        if req.protocols()?.contains(&Self::PROTOCOL) {
            let mut res = ws::Response::from_request(req)?;
            res.set_protocol(Self::PROTOCOL);
            Ok(res)
        } else {
            Err(ws::Error::new(ws::ErrorKind::Protocol, "wrong protocol"))
        }
    }

    fn rpc_on_message(&mut self, msg: ws::Message) -> ws::Result<()> {
        if let Some(rpc) = message::parse_ws(msg) {
            match rpc {
                message::RpcMessage::Request(req) => {
                    trace!("handing off rpc request for handling: {:?}", req);

                    if let Some(res) = self.rpc().handle_rpc_request(req).wait().unwrap() {
                        trace!("got rpc response back from handler: {:?}", res);
                        self.sender().send(json!(res).to_string())?
                    } else {
                        trace!("no rpc response back from handler (is it a notification?)");
                    }
                }
                message::RpcMessage::Response(Response::Single(out)) => {
                    trace!("got a single response");
                    self.handle_response(out)?
                }
                message::RpcMessage::Response(Response::Batch(outsies)) => {
                    trace!("got a batch of {} responses", outsies.len());
                    for out in outsies {
                        self.handle_response(out)?;
                    }
                }
            };
        }

        Ok(())
    }

    fn handle_response(&self, out: Output) -> ws::Result<()> {
        trace!("handling single response output: {:?}", out);

        let id = match out {
            Output::Failure(ref fail) => &fail.id,
            Output::Success(ref succ) => &succ.id,
        };

        trace!("looking up inflight id from response: {:?}", id);

        if let Some(tx) = self.inflight().recall(id) {
            trace!("matched with existing id, sending response through");
            tx.send(Response::Single(out)).expect("Internal comm error");
        }

        Ok(())
    }
}
