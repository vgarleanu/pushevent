use crate::client::Client;
use crate::Event;
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;
use ws::{listen, Sender};

pub(crate) type ServerRef = Arc<Mutex<RefCell<ServerInner>>>;

pub struct Server {
    threads: Vec<thread::JoinHandle<()>>,
    inner: ServerRef,
    tx: mpsc::Sender<Event>,
}

pub(crate) struct ServerInner {
    clients: HashMap<String, Vec<Sender>>,
}

impl Server {
    pub fn new(addr: &'static str) -> Self {
        let inner = Arc::new(Mutex::new(RefCell::new(ServerInner::new())));
        let (tx, rx) = mpsc::channel::<Event>();
        let mut threads = Vec::new();

        let inner_clone = inner.clone();
        threads.push(thread::spawn(move || {
            listen(addr, |s| Client::new(inner_clone.clone(), s))
                .expect("Failed to start ws server");
        }));

        let inner_clone = inner.clone();
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

    pub fn get_tx(&self) -> mpsc::Sender<Event> {
        self.tx.clone()
    }

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

    pub fn add_client(&mut self, res: &str, sender: &Sender) {
        match self.clients.get_mut(&res.to_owned()) {
            Some(x) => x.push(sender.clone()),
            None => {
                let _ = self.clients.insert(res.to_owned(), vec![sender.clone()]);
            }
        }
    }

    pub fn remove_client(&mut self, sender: &Sender) {
        for vec in self.clients.values_mut() {
            vec.retain(|x| x.token() == sender.token())
        }
    }

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
