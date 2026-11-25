use crate::{handler::HandlerArgs, homeassistant::HAClientBuilder, types::Entity};

pub const HANDLER: Handler = Handler {};

pub struct Handler {}
impl crate::handler::Handler for Handler {
    fn mqtt_entities(&self) -> Vec<&dyn Entity> {
        vec![]
    }
    fn start(
        &'static self,
        _tasks: &mut tokio::task::JoinSet<!>,
        _args: HandlerArgs,
        _ha_client: &mut HAClientBuilder,
    ) {
    }
}
