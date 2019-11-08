use serde::Serialize;
use std::thread;
use std::time;
use websocket_event::server::Server;
use websocket_event::Event;
use websocket_event::SerializableEvent;

#[derive(Serialize, Debug)]
struct SimplePushEvent {
    message: String,
}

impl SerializableEvent for SimplePushEvent {
    fn serialize(&self) -> String {
        serde_json::to_string(&self).unwrap()
    }
}

fn main() {
    let server = Server::new("127.0.0.1:3012");
    let tx = server.get_tx();

    loop {
        let msg = Box::new(SimplePushEvent {
            message: String::from("Hello world"),
        });

        let event = Event::new("/hello_world", msg);

        match tx.send(event) {
            Ok(_) => {}
            Err(x) => println!("Err {:?}", x),
        };

        let ten_millis = time::Duration::from_millis(100);
        thread::sleep(ten_millis);
    }
}
