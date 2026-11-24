use log::info;
use tokio::{sync::Mutex, task::JoinSet};

use crate::{
    handler::{HandlerArgs, loopable},
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

impl crate::handler::Handler for MedicationReminder {
    fn entities(&self) -> Vec<&dyn Entity> {
        vec![&self.button, &self.bathroom_sensor]
    }

    fn start(&'static self, tasks: &mut JoinSet<!>, args: HandlerArgs) {
        tasks.spawn(self.handle_daily_reset());
        tasks.spawn(loopable(args, |a| self.handle_button_press(a)));
    }
}
impl MedicationReminder {
    async fn handle_daily_reset(&self) -> ! {
        let ticker = crate::ticker::DailyTicker::new(RESET_TIME);
        loop {
            ticker.next().await;
            info!("Medication state reset");
            self.state.lock().await.medication_taken = false;
        }
    }

    async fn handle_button_press(&self, args: HandlerArgs) -> anyhow::Result<()> {
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
        Ok(())
    }
}
