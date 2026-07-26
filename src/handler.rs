use async_channel::{Receiver, RecvError};
use rumqttc::{Event, Packet};

use crate::types::Message;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("graceful shutdown")]
    Shutdown,
    #[error("{0}")]
    Other(#[from] anyhow::Error),
}
pub type Result<T> = core::result::Result<T, Error>;

#[derive(Clone)]
pub struct HandlerArgs {
    events: Receiver<Event>,
}
impl HandlerArgs {
    pub fn new(events: Receiver<Event>) -> Self {
        HandlerArgs { events }
    }

    /// Receive an event from the queue, handling queue shutdown.
    pub async fn recv(&self) -> Result<Message> {
        loop {
            match self.events.recv().await {
                Ok(Event::Incoming(Packet::Publish(p))) => {
                    if let Ok(m) = Message::deserialize(p.payload) {
                        return Ok(m);
                    }
                }
                Ok(_) => {}
                Err(RecvError) => return Err(Error::Shutdown),
            }
        }
    }
}

#[async_trait::async_trait]
pub trait Handler {
    async fn handle(&self, args: HandlerArgs) -> Result<()>;
}
