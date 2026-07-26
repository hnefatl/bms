use crate::handler::{HandlerArgs, Result};

pub const HANDLER: Handler = Handler {};

pub struct Handler {}
#[async_trait::async_trait]
impl crate::handler::Handler for Handler {
    async fn handle(&self, args: HandlerArgs) -> Result<()> {
        loop {
            let event = args.recv().await?;

            println!("{:?}", event);
        }
    }
}
