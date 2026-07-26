use bytes::Bytes;

use log::trace;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum Message {}

impl Message {
    pub fn deserialize(data: Bytes) -> Result<Message, Bytes> {
        match serde_json::from_slice(&data) {
            Ok(m) => Ok(m),
            Err(e) => {
                trace!("failed to deserialize payload: {}", e);
                Err(data)
            }
        }
    }
}
