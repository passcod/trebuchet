use crate::inflight::Inflight;
use crate::message;
use futures::{Future, sync::oneshot::Receiver};
use jsonrpc_core::{IoHandler, Output, Params, Response};

pub trait RpcHandler {
    const PROTOCOL: &'static str;

    fn inflight(&self) -> &Inflight;
    fn rpc(&self) -> &IoHandler;
    fn sender(&self) -> &ws::Sender;

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

    fn notify(&self, method: String, params: Params, binary: Option<&[u8]>) -> ws::Result<()> {
        let msg: ws::Message = match binary {
            None => message::notification(method, params).into(),
            Some(raw) => message::add_binary(message::notification(method, params), raw).into(),
        };
        self.sender().send(msg)
    }

    fn rpc_build_request(&self, url: &url::Url) -> ws::Result<ws::Request> {
        let mut req = ws::Request::from_url(url)?;
        req.add_protocol(Self::PROTOCOL);
        Ok(req)
    }

    fn rpc_on_response(&self, res: &ws::Response) -> ws::Result<()> {
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
                message::RpcMessage::Response(res) => self.handle_response(res)?,
            };
        }

        Ok(())
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

                Ok(())
            }
            Response::Single(out) => self.handle_response_out(out),
        }
    }
}
