use log::info;
use tokio::sync::Mutex;
use tokio_util::task::AbortOnDropHandle;

use crate::{
    handler::HandlerArgs,
    types::{AqaraTemperatureSensor, Entity, IKEAShortcutButton, IKEAShortcutButtonAction},
};

const RESET_TIME: jiff::civil::Time = jiff::civil::Time::constant(2, 0, 0, 0);

pub static MEDICATION_REMINDER: MedicationReminder = MedicationReminder {
    state: Mutex::const_new(State {
        medication_taken: false,
    }),
    button: IKEAShortcutButton::new("zigbee2mqtt/medication_switch"),
    bathroom_sensor: AqaraTemperatureSensor::new("zigbee2mqtt/bathroom"),
};

pub struct MedicationReminder {
    state: Mutex<State>,
    button: IKEAShortcutButton,
    bathroom_sensor: AqaraTemperatureSensor,
}
#[derive(Debug)]
pub struct State {
    medication_taken: bool,
}

#[async_trait::async_trait]
impl crate::handler::Handler for MedicationReminder {
    fn entities(&self) -> Vec<&dyn Entity> {
        vec![&self.button, &self.bathroom_sensor]
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

        // TODO: this is getting unwieldy: maybe neater to do callbacks/threads on entity events?
        loop {
            let p = args.recv().await?;
            match self.button.get_payload(&p).map(|p| p.action) {
                Some(IKEAShortcutButtonAction::On) => {
                    info!("Medication taken");
                    self.state.lock().await.medication_taken = true;
                }
                Some(IKEAShortcutButtonAction::BrightnessMoveUp) => {
                    info!("Medication reminder unimplemented");
                }
                _ => {}
            };
            match self.bathroom_sensor.get_payload(&p) {
                Some(s) => info!("Bathroom sensor: {:?}", s),
                _ => {}
            }
        }
    }
}
