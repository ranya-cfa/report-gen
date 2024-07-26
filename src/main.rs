mod global_state;
mod context;

use global_state::{GlobalState, Incidence, Death};
use context::Context;
use std::thread;
use std::sync::{Arc, Mutex};

fn main() {
    let global_state = Arc::new(Mutex::new(GlobalState::new()));

    {
        let mut gs = global_state.lock().unwrap();
        gs.register_report_type::<Incidence>("incidence.csv");
        gs.register_report_type::<Death>("death.csv");
        gs.start_consumer_thread();
    }

    let context1 = Context::new(&global_state);
    let context2 = Context::new(&global_state);

    context1.add_report(Incidence { timestamp: "2024-07-26".to_string(), new_cases: 42 });
    context2.add_report(Death { timestamp: "2024-07-26".to_string(), deaths: 1 });

    {
        let mut gs = global_state.lock().unwrap();
        gs.join_consumer_thread();
    }
}
