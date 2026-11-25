use async_channel::{Receiver, RecvError};
use log::error;
use rumqttc::{AsyncClient, Event, Packet, Publish};
use tokio::task::JoinSet;

use crate::{homeassistant::HAClientBuilder, types::Entity};

#[derive(Clone)]
pub struct HandlerArgs {
    pub mqtt_client: AsyncClient,
    mqtt_events: Receiver<Event>,
}
impl HandlerArgs {
    pub fn new(mqtt_client: AsyncClient, mqtt_events: Receiver<Event>) -> Self {
        HandlerArgs { mqtt_client, mqtt_events }
    }

    /// Receive a publish from the queue, handling queue shutdown.
    pub async fn recv(&self) -> Publish {
        loop {
            match self.mqtt_events.recv().await {
                Ok(Event::Incoming(Packet::Publish(p))) => {
                    return p;
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
    fn mqtt_entities(&self) -> Vec<&dyn Entity>;

    fn start(&'static self, tasks: &mut JoinSet<!>, args: HandlerArgs, ha_client: &mut HAClientBuilder);
}

/// Run the given function until cancelled, printing errors and restarting.
pub async fn loopable<T: Clone, F: AsyncFn(T) -> anyhow::Result<()>>(t: T, f: F) -> ! {
    loop {
        if let Err(e) = f(t.clone()).await {
            error!("{}", e)
        }
    }
}
