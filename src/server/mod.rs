use crate::client::Client;
use crate::message::{parse_binary, parse_plain, Rpc};
use futures::Future;
use jsonrpc_core::IoHandler;

pub use self::for_worker::*;

mod for_worker;

pub trait Server: Client {
    fn rpc(&self) -> &IoHandler;

    fn server_on_request(&mut self, req: &ws::Request, protocol: &str) -> ws::Result<ws::Response> {
        if req.protocols()?.contains(&protocol) {
            let mut res = ws::Response::from_request(req)?;
            res.set_protocol(protocol);
            Ok(res)
        } else {
            Err(ws::Error::new(ws::ErrorKind::Protocol, "wrong protocol"))
        }
    }

    fn server_on_message(&mut self, msg: ws::Message) -> ws::Result<()> {
        if let Some(rpc) = match msg {
            ws::Message::Text(string) => {
                trace!("string message received: {:?}", string);
                parse_plain(&string)
            }
            ws::Message::Binary(raw) => {
                trace!("raw message received: {:?}", raw);
                parse_binary(&raw)
            }
        } {
            match rpc {
                Rpc::Request(req) => {
                    trace!("handing off rpc request for handling: {:?}", req);

                    if let Some(res) = self.rpc().handle_rpc_request(req).wait().unwrap() {
                        trace!("got rpc response back from handler: {:?}", res);
                        self.sender().send(json!(res).to_string())?
                    } else {
                        trace!("no rpc response back from handler (is it a notification?)");
                    }
                }
                Rpc::Response(res) => self.handle_response(res)?,
            };
        }
        Ok(())
    }
}
