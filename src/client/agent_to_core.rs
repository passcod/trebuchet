use crossbeam_channel as cc;

pub type MessagePassthru = (cc::Sender<ws::Message>, cc::Receiver<ws::Message>);

/// Client from Agent to Core.
pub struct AgentCoreClient {
    sender: ws::Sender,
    passthru: MessagePassthru,
}

impl AgentCoreClient {
    fn create(sender: ws::Sender) -> Self {
        let passthru = cc::unbounded();
        Self { sender, passthru }
    }

    fn corepass(&self) -> MessagePassthru {
        (self.passthru.0.clone(), self.passthru.1.clone())
    }
}

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
