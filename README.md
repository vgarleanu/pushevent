# PushEvent

PushEvent is a simple event dispatch library built on top of ws-rs, that allows you to dispatch events to clients based on what resource they are subscribed to.

```rust
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
            message: String::from("Hello world"),
        });

        // The previous message event is encapsulated in our Event struct to which we supply two
        // arguments, the path/resource subscribers we would like to target ("/hello_world") and
        // our message event struct instance which implements SerializableEvent.
        let event = Event::new("/hello_world", msg);

        // The event is then sent over the tx channel provided by our server instance
        match tx.send(event) {
            Ok(_) => {}
            Err(x) => println!("Err {:?}", x),
        };

        let millis = time::Duration::from_millis(100);
        thread::sleep(millis);
    }
}
```

Accessing `127.0.0.1:3012/hello_world`, for example, will subscribe you to that route, yielding you with a
message every 100ms which should be `{'message': 'Hello world'}`. This can be further used to build and publish more complex events.

## Documentation

PushEvent is simple and straightforward to use:

  * [Examples]: Holds examples of how to use this library.
  * [API Documentation]: The "rustdocs".

[Examples]: examples/
[API Documentation]: https://docs.rs/pushevent/


### Examples

Rocket ships with an extensive number of examples in the `examples/` directory
which can be compiled and run with Cargo. For instance, the following sequence
of commands builds and runs the `Hello, world!` example:

```
cd examples/simple
cargo run
```

You should see `{'message': 'Hello world'}` by accessing `ws://127.0.0.1:3012`.

## Testing

To test this library simply run `cargo test`.

## Contributing

Contributions are absolutely, positively welcome and encouraged! Contributions
come in many forms. You could:

  1. Submit a feature request or bug report as an [issue].
  2. Ask for improved documentation as an [issue].
  3. Contribute code via [merge requests].

[issue]: https://gitlab.com/vgarleanu/pushevent/issues
[merge requests]: https://gitlab.com/vgarleanu/pushevent/merge_requests

All pull requests are code reviewed and tested by the CI. Note that unless you
explicitly state otherwise, any contribution intentionally submitted for
inclusion in PushEvent by you shall be licensed under the MIT License 
without any additional terms or conditions.

## License

PushEvent is licensed under the MIT License ([LICENSE.md](LICENSE.md) or http://opensource.org/licenses/MIT)
