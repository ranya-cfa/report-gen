mod global_state;
mod context;

use global_state::{GlobalState, Incidence, Death};
use context::Context;
use std::sync::{Arc, Mutex};

fn main() {
    let global_state = Arc::new(Mutex::new(GlobalState::new()));
    // Adding reports during initialization
    {
        let mut state = global_state.lock().unwrap();
        state.add_report::<Incidence>("incidence_report.csv");
        state.add_report::<Death>("death_report.csv");
    }

    let context1 = Context::new(&global_state);
    let context2 = Context::new(&global_state);

    // Release report items
    context1.release_report_item(Incidence {
        timestamp: "2023-06-26 0".to_string(),
        new_cases: 150,
    });
    context2.release_report_item(Incidence {
        timestamp: "2023-06-26 1".to_string(),
        new_cases: 160,
    });

    context1.release_report_item(Death {
        timestamp: "2023-06-26 0".to_string(),
        deaths: 5,
    });
    
    //context1.execute();
    //context2.execute()
}