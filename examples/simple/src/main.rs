use pushevent::server::Server;
use pushevent::Event;
use pushevent::SerializableEvent;
use serde::Serialize;
use std::thread;
use std::time;

/// Basic event struct which serializes with serde to json.
#[derive(Serialize, Debug)]
struct SimplePushEvent {
    message: String,
}

impl SerializableEvent for SimplePushEvent {
    /// Serialize method used as a intermediary to serialize the struct into a json string and
    /// return it.
    fn serialize(&self) -> String {
        serde_json::to_string(&self).unwrap()
    }
}

fn main() {
    // Server is started on localhost with port 3012
    let server = Server::new("127.0.0.1:3012");
    let tx = server.get_tx();

    loop {
        // We create a new boxed instance of our SimplePushEvent struct with whatever message
        // inside.
        let msg = Box::new(SimplePushEvent {
            message: "Hello world".to_string(),
        });

        // The previous message event is encapsulated in our Event struct to which we supply two
        // arguments, the path/resource subscribers we would like to target ("/hello_world") and
        // our message event struct instance which implements SerializableEvent.
        let event = Event::new("/hello_world".to_string(), msg);

        // The event is then sent over the tx channel provided by our server instance
        match tx.send(event) {
            Ok(_) => {}
            Err(x) => println!("Err {:?}", x),
        };

        let ten_millis = time::Duration::from_millis(100);
        thread::sleep(ten_millis);
    }
}
