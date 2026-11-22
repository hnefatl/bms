use crate::{
    handler::{HandlerArgs, Result},
    types::Entity,
};

pub const HANDLER: Handler = Handler {};

pub struct Handler {}
#[async_trait::async_trait]
impl crate::handler::Handler for Handler {
    fn entities(&self) -> Vec<&dyn Entity> {
        vec![]
    }
    async fn handle(&'static self, _args: HandlerArgs) -> Result<()> {
        unimplemented!()
    }
}
