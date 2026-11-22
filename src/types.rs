use log::error;
use rumqttc::Publish;
use serde::{Deserialize, de::DeserializeOwned};
use std::marker::PhantomData;

/// Represents an entity that has a topic and an associated event type.
pub trait Entity {
    fn topic(&self) -> &str;
}

pub struct EntityImpl<Payload: DeserializeOwned> {
    p: PhantomData<Payload>,
    topic: &'static str,
}
impl<Payload: DeserializeOwned> EntityImpl<Payload> {
    pub const fn new(topic: &'static str) -> Self {
        Self {
            p: PhantomData,
            topic,
        }
    }

    /// Check if the topic matches, and parse the payload if so.
    pub fn get_payload(&self, p: &Publish) -> Option<Payload> {
        if p.topic != self.topic() {
            return None;
        }
        match serde_json::from_slice(&p.payload) {
            Ok(e) => Some(e),
            Err(e) => {
                error!("failed to deserialize payload ({}): {:?}", e, p.payload);
                None
            }
        }
    }
}
impl<E: DeserializeOwned> Entity for EntityImpl<E> {
    fn topic(&self) -> &str {
        self.topic
    }
}

pub type IKEAShortcutButton = EntityImpl<IKEAShortcutButtonPayload>;
#[derive(Deserialize, Debug)]
pub struct IKEAShortcutButtonPayload {
    pub action: IKEAShortcutButtonAction,
}
#[derive(Deserialize, Debug)]
pub enum IKEAShortcutButtonAction {
    #[serde(rename = "on")]
    On,
    #[serde(rename = "brightness_move_up")]
    BrightnessMoveUp,
    #[serde(rename = "brightness_stop")]
    BrightnessStop,
}

pub type AqaraTemperatureSensor = EntityImpl<AqaraTemperatureSensorPayload>;
#[derive(Deserialize, Debug)]
pub struct AqaraTemperatureSensorPayload {
    temperature: f32,
    humidity: f32,
    pressure: f32,
}