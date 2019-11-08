#![feature(result_map_or_else)]
pub mod client;
pub mod server;

pub trait SerializableEvent: Sync + Send + 'static {
    fn serialize(&self) -> String;
}

pub struct Event {
    res: &'static str,
    inner: Box<dyn SerializableEvent>,
}

impl Event {
    pub fn new(res: &'static str, inner: Box<dyn SerializableEvent>) -> Self {
        Self { res, inner: inner }
    }

    pub fn get_res(&self) -> String {
        self.res.to_owned()
    }

    pub fn build(&self) -> String {
        self.inner.serialize()
    }
}
