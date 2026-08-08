use async_channel::{Receiver, RecvError};
use bytes::Bytes;
use rumqttc::{AsyncClient, Event, Packet};

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
    pub client: AsyncClient,
    events: Receiver<Event>,
}
impl HandlerArgs {
    pub fn new(client: AsyncClient, events: Receiver<Event>) -> Self {
        HandlerArgs { client, events }
    }

    /// Receive an event from the queue, handling queue shutdown.
    pub async fn recv(&self) -> Result<Message> {
        loop {
            if let Ok(m) = Message::deserialize(self.recv_raw().await?) {
                return Ok(m);
            }
        }
    }
    pub async fn recv_raw(&self) -> Result<Bytes> {
        loop {
            match self.events.recv().await {
                Ok(Event::Incoming(Packet::Publish(p))) => {
                    return Ok(p.payload);
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
