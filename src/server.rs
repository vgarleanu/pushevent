use crate::client::Client;
use crate::Event;
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;
use ws::{listen, Sender};

pub(crate) type ServerRef = Arc<Mutex<RefCell<ServerInner>>>;

/// The main server struct that gets returned when a ws server is opened.
/// It encapsulates a vector of thread join handles, which holds mainly our Websocket server
/// thread and a thread which receives messages from our mpsc channel. It also holds our inner
/// ServerRef which you probably don't need to touch.
pub struct Server {
    threads: Vec<thread::JoinHandle<()>>,
    inner: ServerRef,
    tx: mpsc::Sender<Event>,
}

/// This is the inner server structs which holds a HashMap of all clients subscribed to a specific
/// route. It is used internally to filter to whom we publish events.
pub(crate) struct ServerInner {
    clients: HashMap<String, Vec<Sender>>,
}

impl Server {
    /// Returns a server instance. As soon as this method is called, a websocket server is opened
    /// and a thread will start accepting events to be published.
    /// # Arguments
    ///
    /// * `addr` - Static string slice which holds the address on which to open the websocket
    /// server
    ///
    /// # Example
    /// ```
    /// let server = Server::new("127.0.0.1:3012");
    /// ```
    pub fn new(addr: &'static str) -> Self {
        let inner = Arc::new(Mutex::new(RefCell::new(ServerInner::new())));
        let (tx, rx) = mpsc::channel::<Event>();
        let mut threads = Vec::new();

        let inner_clone = inner.clone();

        // This thread handles our websocket server
        threads.push(thread::spawn(move || {
            listen(addr, |s| Client::new(inner_clone.clone(), s))
                .expect("Failed to start ws server");
        }));

        let inner_clone = inner.clone();
        // This thread handles our mpsc rx channel.
        // It awaits for new messages to be pushed, it then locks the inner ServerInner mutex,
        // serializes our event and broadcasts it to the specified resource.
        // All events received over this channel are of Event type.
        threads.push(thread::spawn(move || {
            for event in rx.iter() {
                inner_clone
                    .lock()
                    .unwrap()
                    .borrow_mut()
                    .broadcast(&event.get_res(), event.build());
            }
        }));

        Self { threads, inner, tx }
    }

    /// Clones and returns a mpsc tx channel through which we can send events of [`Event`](Event)
    /// type.
    ///
    /// # Example
    /// ```
    /// use std::thread;
    ///
    /// let server = Server::new("127.0.0.1:3012");
    ///
    /// loop {
    ///     let tx = server.get_tx();
    ///     let _ = std::thread::spawn(move || {
    ///         tx.send(...);
    ///     });
    /// }
    /// ```
    pub fn get_tx(&self) -> mpsc::Sender<Event> {
        self.tx.clone()
    }

    /// This method cleans up all of our threads and should be called on exit of your main
    /// application.
    /// It drains all threads from self.threads and tries to join them.
    ///
    /// # Example
    /// ```
    /// let server = Server::new("127.0.0.1:3012");
    /// server.join_threads();
    /// ```
    pub fn join_threads(&mut self) {
        for thread in self.threads.drain(0..) {
            thread.join().unwrap();
        }
    }
}

impl ServerInner {
    pub fn new() -> Self {
        ServerInner {
            clients: HashMap::new(),
        }
    }

    /// Method used internally to add a client to the hashmap based on the route they have
    /// connected to.
    /// # Arguments
    /// * `res` - Resource path to which the client has connected.
    /// * `sender` - A Sender is a client that has connected to our server
    ///
    /// # Example
    /// ```
    /// let inner = ServerInner::new();
    /// let sender = Sender {...};
    ///
    /// inner.add_client("/hello", sender);
    /// assert_eq!(inner.clients.len(), 1usize);
    /// ```
    pub fn add_client(&mut self, res: &str, sender: &Sender) {
        match self.clients.get_mut(&res.to_owned()) {
            Some(x) => x.push(sender.clone()),
            None => {
                let _ = self.clients.insert(res.to_owned(), vec![sender.clone()]);
            }
        }
    }

    /// Method used internally to removed clients that have disconnected from the global hashmap so
    /// that events stop being published to them.
    ///
    /// # Arguments
    /// * `sender` - A Sender is a client that has connected to our server
    ///
    /// # Example
    /// ```
    /// let inner = ServerInner::new();
    /// let sender = Sender {...};
    ///
    /// inner.add_client("/hello", sender);
    /// assert_eq!(inner.clients.len(), 1usize);
    ///
    /// inner.remove_client(sender);
    /// assert_eq!(inner.clients.len(), 0usize);
    /// ```
    pub fn remove_client(&mut self, sender: &Sender) {
        for vec in self.clients.values_mut() {
            vec.retain(|x| x.token() == sender.token())
        }
    }

    /// Method used internally to broadcast messages to clients subscribed to a specific route. The
    /// message will only be broadcast once to all connected clients.
    ///
    /// # Arguments
    /// * `res` - String slice which holds the resource path we would like to publish events to.
    /// * `msg` - String which holds the message we wish to publish.
    ///
    /// # Example
    /// ```
    /// let inner = ServerInner::new();
    /// inner.broadcast("/hello", "Hello World");
    /// ```
    pub fn broadcast(&self, res: &str, msg: String) {
        let _ = self.clients.get(res).map(|x| {
            for y in x {
                y.send(msg.clone()).unwrap()
            }
        });
    }
}

impl Default for ServerInner {
    fn default() -> Self {
        Self::new()
    }
}
