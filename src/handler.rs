use async_channel::{Receiver, RecvError};
use rumqttc::{AsyncClient, Event, Packet, Publish};

use crate::types::Entity;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("graceful shutdown")]
    Shutdown,
    #[error("{0}")]
    Other(#[from] anyhow::Error),
    #[error("{0}")]
    InvalidStateTransition(#[from] rust_fsm::TransitionImpossibleError),
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

    /// Receive a publish from the queue, handling queue shutdown.
    pub async fn recv(&self) -> Result<Publish> {
        loop {
            match self.events.recv().await {
                Ok(Event::Incoming(Packet::Publish(p))) => {
                    return Ok(p);
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

    fn entities(&self) -> Vec<&dyn Entity>;
}
