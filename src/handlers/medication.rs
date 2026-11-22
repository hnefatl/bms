use log::info;
use tokio::sync::Mutex;
use tokio_util::task::AbortOnDropHandle;

use crate::{
    handler::HandlerArgs,
    types::{Entity, IKEAShortcutButton, IKEAShortcutButtonAction},
};

const RESET_TIME: jiff::civil::Time = jiff::civil::Time::constant(2, 0, 0, 0);

pub static HANDLER: Handler = Handler {
    state: Mutex::const_new(State {
        medication_taken: false,
    }),
    button: IKEAShortcutButton::new("zigbee2mqtt/medication_switch"),
};

pub struct Handler {
    state: Mutex<State>,
    button: IKEAShortcutButton,
}
#[derive(Debug)]
pub struct State {
    medication_taken: bool,
}

#[async_trait::async_trait]
impl crate::handler::Handler for Handler {
    fn entities(&self) -> Vec<&dyn Entity> {
        vec![&self.button]
    }

    async fn handle(&'static self, args: HandlerArgs) -> crate::handler::Result<()> {
        let _handle = AbortOnDropHandle::new(tokio::spawn(async {
            let ticker = crate::ticker::DailyTicker::new(RESET_TIME);
            loop {
                ticker.next().await;
                info!("Medication state reset");
                self.state.lock().await.medication_taken = false;
            }
        }));

        loop {
            let Some(e) = self.button.get_payload(args.recv().await?) else {
                continue;
            };
            match e.action {
                IKEAShortcutButtonAction::On => {
                    info!("Medication taken");
                    self.state.lock().await.medication_taken = true;
                }
                IKEAShortcutButtonAction::BrightnessMoveUp => {
                    info!("Medication reminder unimplemented");
                }
                IKEAShortcutButtonAction::BrightnessStop => {}
            };
        }
    }
}
