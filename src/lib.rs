#![feature(result_map_or_else)]
pub mod client;
pub mod server;

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
    res: &'static str,
    inner: Box<dyn SerializableEvent>,
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
    pub fn new(res: &'static str, inner: Box<dyn SerializableEvent>) -> Self {
        Self { res, inner }
    }

    /// Returns the resource path that this event targets. Usually used internally and rarely
    /// useful.
    /// # Example
    /// ```
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
    /// ```
    pub fn get_res(&self) -> String {
        self.res.to_owned()
    }

    /// Serializes and returns the inner event/message.
    /// # Example
    /// ```
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
        self.inner.serialize()
    }
}
