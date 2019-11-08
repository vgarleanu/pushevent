use crate::server::ServerRef;
use ws::{CloseCode, Handler, Handshake, Message, Request, Response, Result as WsResult, Sender};

/// Client structure which encapsulates a Sender with some extra info like resource and our
/// ServerRef.
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
    /// Methods called by ws-rs internally whenever a new request is made.
    /// The method locks our ServerRef and adds a new client with the resource requested by which
    /// we can filter later on.
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

    // Method called by ws-rs internally whenever a client disconnected.
    // Method locks our ServerRef and removes the client from our list.
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
