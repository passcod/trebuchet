/// Client from Agent to Core.
pub struct AgentCoreClient {}

impl ws::Handler for AgentCoreClient {
    fn build_request(&mut self, url: &url::Url) -> ws::Result<ws::Request> {
        let mut req = ws::Request::from_url(url)?;
        req.add_protocol("armstrong/agent");
        Ok(req)
    }

    fn on_response(&mut self, res: &ws::Response) -> ws::Result<()> {
        match res.protocol()? {
            Some("armstrong/agent") => Ok(()),
            _ => Err(ws::Error::new(ws::ErrorKind::Protocol, "wrong protocol")),
        }
    }

    fn on_open(&mut self, _shake: ws::Handshake) -> ws::Result<()> {
        info!("connected to core");
        // send greeting
        Ok(())
    }

    fn on_message(&mut self, _msg: ws::Message) -> ws::Result<()> {
        // Handle server messages
        Ok(())
    }

    fn on_shutdown(&mut self) {
        // info!() something out
    }
}
