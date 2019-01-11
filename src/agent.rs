use crate::proto::Worker;
use crate::system::System;
use std::collections::HashMap;

pub struct Agent {
    system: System,

    // worker name -> definition
    workers: HashMap<String, Worker>,
    // workers comm with the agent through JSON RPC over Websocket.
    // the agent communicates with the daemon through the same.
    // i.e. agents run a server and a client,
    //      workers run a client,
    //      daemon runs a server.
    //
    // Why JSON RPC? Simple, lightweight, well-established, can be hand-written in a pinch
    // Why Websocket? Duplex, inspectable, trivial to secure, can be used from browsers as-is
}

impl ws::Handler for Agent {
    // client
    fn build_request(&mut self, url: &url::Url) -> ws::Result<ws::Request> {
        let mut req = ws::Request::from_url(url)?;
        req.add_protocol("armstrong/agent");
        Ok(req)
    }

    // client
    fn on_response(&mut self, res: &ws::Response) -> ws::Result<()> {
        // check server protocol and deets
        match res.protocol()? {
            Some("armstrong/agent") => Ok(()),
            _ => Err(ws::Error::new(ws::ErrorKind::Protocol, "wrong protocol"))
        }
    }

    // server
    fn on_request(&mut self, req: &ws::Request) -> ws::Result<ws::Response> {
        if req.protocols()?.contains(&&"armstrong/worker") {
            let mut res = ws::Response::from_request(req)?;
            res.set_protocol("armstrong/worker");
            Ok(res)
        } else {
            Err(ws::Error::new(ws::ErrorKind::Protocol, "wrong protocol"))
        }
    }

    // both
    fn on_open(&mut self, shake: ws::Handshake) -> ws::Result<()> {
        // info!() something out, send greeting
        Ok(())
    }

    // both
    fn on_message(&mut self, msg: ws::Message) -> ws::Result<()> {
        // Handle server messages
        Ok(())
    }

    // both
    fn on_shutdown(&mut self) {
        // info!() something out
    }
}
