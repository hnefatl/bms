use crate::{handler::HandlerArgs, types::Entity};

pub const HANDLER: Handler = Handler {};

pub struct Handler {}
impl crate::handler::Handler for Handler {
    fn entities(&self) -> Vec<&dyn Entity> {
        vec![]
    }
    fn start(&'static self, _tasks: &mut tokio::task::JoinSet<!>, _args: HandlerArgs) {}
}
