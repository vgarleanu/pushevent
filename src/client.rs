use crate::server::ServerRef;
use ws::{CloseCode, Handler, Handshake, Message, Request, Response, Result as WsResult, Sender};

#[derive(Clone)]
pub struct Client {
    server: ServerRef,
    sender: Sender,
    resource: Option<String>,
}

impl Client {
    pub(crate) fn new(server: ServerRef, sender: Sender) -> Self {
        Self {
            server,
            sender,
            resource: None,
        }
    }
}

impl Handler for Client {
    fn on_request(&mut self, req: &Request) -> WsResult<(Response)> {
        self.server
            .lock()
            .unwrap()
            .borrow_mut()
            .add_client(req.resource(), &self.sender);

        Ok(Response::from_request(req)?)
    }

    fn on_open(&mut self, _: Handshake) -> WsResult<()> {
        Ok(())
    }

    fn on_close(&mut self, _: CloseCode, _: &str) {
        self.server
            .lock()
            .unwrap()
            .borrow_mut()
            .remove_client(&self.sender);
    }

    fn on_message(&mut self, _: Message) -> WsResult<()> {
        Ok(())
    }
}
