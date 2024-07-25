mod global_state;
mod context;

use global_state::{GlobalState, Incidence, Death};
use context::Context;
use crate::global_state::GLOBAL_STATE;
use std::thread;
use std::sync::{Arc, Mutex};

fn main() {
    let mut global_state = GLOBAL_STATE.lock().unwrap();

    global_state.register_report_type::<Incidence>("incidence_report.csv");
    global_state.register_report_type::<Death>("death_report.csv");

    global_state.start_consumer_thread(); // allows multiple contexts to share same global state

    let global_state = GLOBAL_STATE.clone();
    let context1 = Context::new(&global_state);
    let context2 = Context::new(&global_state);

    let producer1 = thread::spawn(move || {
        context1.add_report(Incidence {
            timestamp: "2023-06-26".to_string(),
            new_cases: 150,
        });
        context1.add_report(Death {
            timestamp: "2023-06-26".to_string(),
            deaths: 5,
        });
    });

    let producer2 = thread::spawn(move || {
        context2.add_report(Incidence {
            timestamp: "2023-06-27".to_string(),
            new_cases: 200,
        });
        context2.add_report(Death {
            timestamp: "2023-06-27".to_string(),
            deaths: 8,
        });
    });

    producer1.join().unwrap();
    producer2.join().unwrap();

    {
        let mut global_state = GLOBAL_STATE.lock().unwrap();
        global_state.join_consumer_thread();
    } // Ensure the consumer thread is joined before exiting
}
