use std::{
    collections::HashMap,
    env,
    io::Error as IoError,
    net::SocketAddr,
    sync::{Arc, Mutex},
};

use futures_channel::mpsc::{unbounded, UnboundedSender};
use futures_util::{future, pin_mut, stream::TryStreamExt, StreamExt};

use tokio::net::{TcpListener, TcpStream};
use tungstenite::protocol::Message;

/// SerializableEvent denotes structs that are able to serialize to some String.
/// This is used as mainly a marker trait, underneath serialize you most likely would want to use
/// serde.
pub trait SerializableEvent: Sync + Send + 'static {
    /// Returns a String of the serialized object
    fn serialize(&self) -> String;
}

/// Base Event struct which can be sent across a channel provided by
/// [`Server::get_tx`](server::Server::get_tx).
/// This struct encapsulates a inner trait object and res which is the resource we want to target.
pub struct Event {
    inner: String,
}

impl Event {
    /// Returns a Event instance.
    /// # Arguments
    ///
    /// * `res` - A string slice that holds the resource we want to target
    /// * `inner` - A boxed trait object that can serialize to a string.
    ///
    /// # Example
    /// ```
    /// use pushevent::{Event, SerializableEvent};
    /// struct Message;
    ///
    /// impl SerializableEvent for Message {
    ///     fn serialize(&self) -> String {
    ///         String::from("Hello world")
    ///     }
    /// }
    ///
    /// let message = Box::new(Message);
    /// let new_event = Event::new("/events/message", message);
    ///
    /// assert_eq!(new_event.get_res(), String::from("/events/message"));
    /// assert_eq!(new_event.build(), String::from("Hello world"));
    /// ```
    pub fn new(inner: impl SerializableEvent) -> Self {
        Self {
            inner: inner.serialize(),
        }
    }

    /// Serializes and returns the inner event/message.
    /// # Example
    /// ```
    /// use pushevent::{Event, SerializableEvent};
    /// struct Message;
    ///
    /// impl SerializableEvent for Message {
    ///     fn serialize(&self) -> String {
    ///         String::from("Hello world")
    ///     }
    /// }
    ///
    /// let message = Box::new(Message);
    /// let new_event = Event::new("/events/message", message);
    /// assert_eq!(new_event.build(), String::from("Hello world"));
    /// ```
    pub fn build(&self) -> String {
        self.inner.clone()
    }
}

type Tx = UnboundedSender<Message>;
type PeerMap = Arc<Mutex<HashMap<SocketAddr, Tx>>>;

pub type EventTx = UnboundedSender<Event>;

async fn handle_connection(peer_map: PeerMap, raw_stream: TcpStream, addr: SocketAddr) {
    let ws_stream = tokio_tungstenite::accept_async(raw_stream)
        .await
        .expect("Error during the websocket handshake occurred");

    // Insert the write part of this peer to the peer map.
    let (tx, rx) = unbounded();
    peer_map.lock().unwrap().insert(addr, tx);

    let (outgoing, incoming) = ws_stream.split();

    let broadcast_incoming = incoming.try_for_each(|msg| {
        future::ok(())
    });

    let receive_from_others = rx.map(Ok).forward(outgoing);

    pin_mut!(broadcast_incoming, receive_from_others);
    future::select(broadcast_incoming, receive_from_others).await;

    peer_map.lock().unwrap().remove(&addr);
}

pub async fn build() -> Result<UnboundedSender<Event>, ()> {
    let addr = "127.0.0.1:3012".to_string();
    let state = PeerMap::new(Mutex::new(HashMap::new()));
    let (tx, rx) = unbounded();

    let listener = TcpListener::bind(&addr).await.expect("failed to bind");

    let state_clone = state.clone();

    tokio::spawn(async move {
        while let Ok((stream, addr)) = listener.accept().await {
            tokio::spawn(handle_connection(state_clone.clone(), stream, addr));
        }
    });

    let broadcast_incoming = rx.for_each(move |msg: Event| {
        let peers = state.lock().unwrap();

        // We want to broadcast the message to everyone except ourselves.
        let broadcast_recipients =
            peers.iter().map(|(_, ws_sink)| ws_sink);

        for recp in broadcast_recipients {
            recp.unbounded_send(Message::text(msg.build()));
        }

        future::ready(())
    });

    tokio::spawn(broadcast_incoming);

    Ok(tx)
}
