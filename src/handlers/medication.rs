use rumqttc::QoS;

use crate::handler::{HandlerArgs, Result};

pub const HANDLER: Handler = Handler {};

pub struct Handler {}
#[async_trait::async_trait]
impl crate::handler::Handler for Handler {
    async fn handle(&self, args: HandlerArgs) -> Result<()> {
        // TODO: better failure mode here? fallible initial setup step?
        args.client
            .subscribe("zigbee2mqtt/medication_switch/action", QoS::AtLeastOnce)
            .await
            .unwrap();
        loop {
            let msg = args.recv_raw().await?;

            println!("{:?}", msg);
        }
    }
}
