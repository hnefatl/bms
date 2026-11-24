use async_channel::{Receiver, RecvError};
use log::error;
use rumqttc::{AsyncClient, Event, Packet, Publish};
use tokio::task::JoinSet;

use crate::types::Entity;

#[derive(thiserror::Error, Debug)]
pub enum Error {
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

    /// Receive a publish from the queue, handling queue shutdown.
    pub async fn recv(&self) -> Result<Publish> {
        loop {
            match self.events.recv().await {
                Ok(Event::Incoming(Packet::Publish(p))) => {
                    return Ok(p);
                }
                Ok(_) => {}
                // If the channel is closed, we're terminating.
                // Wait ~indefinitely for this future to get aborted.
                Err(RecvError) => std::future::pending().await,
            }
        }
    }
}

pub trait Handler {
    fn entities(&self) -> Vec<&dyn Entity>;

    fn start(&'static self, tasks: &mut JoinSet<!>, args: HandlerArgs);
}

/// Run the given function until cancelled, printing errors and restarting.
pub async fn loopable<T: Clone, F: AsyncFn(T) -> anyhow::Result<()>>(t: T, f: F) -> ! {
    loop {
        if let Err(e) = f(t.clone()).await {
            error!("{}", e)
        }
    }
}
