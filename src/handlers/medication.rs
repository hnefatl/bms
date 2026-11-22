use log::info;
use rust_fsm::*;
use tokio::sync::Mutex;

use crate::{
    handler::HandlerArgs,
    types::{Entity, IKEAShortcutButton, IKEAShortcutButtonAction},
};

pub static HANDLER: Handler = Handler {
    machine: Mutex::const_new(medication::StateMachine::new()),
    button: IKEAShortcutButton::new("zigbee2mqtt/medication_switch"),
};

state_machine! {
    #[derive(Debug)]
    medication(MedsNotTaken)

    MedsNotTaken(MedsTaken) => MedsTaken,
    MedsTaken(MedsTaken) => MedsTaken,
    MedsTaken(DayReset) => MedsNotTaken,
}

pub struct Handler {
    machine: Mutex<medication::StateMachine>,
    button: IKEAShortcutButton,
}

#[async_trait::async_trait]
impl crate::handler::Handler for Handler {
    fn entities(&self) -> Vec<&dyn Entity> {
        vec![&self.button]
    }

    async fn handle(&self, args: HandlerArgs) -> crate::handler::Result<()> {
        // TODO: ticker and chrono.
        loop {
            let Some(e) = self.button.get_payload(args.recv().await?) else {
                continue;
            };
            let output = {
                let mut machine = self.machine.lock().await;
                match e.action {
                    IKEAShortcutButtonAction::On => {
                        info!("Medication taken");
                        machine.consume(&medication::Input::MedsTaken)?
                    }
                    _ => None,
                }
            };
        }
    }
}
