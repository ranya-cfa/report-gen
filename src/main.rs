mod global_state;
mod context;

use global_state::{GlobalState, Incidence, Death};
use context::Context;
use std::sync::{Arc, Mutex};
use serde_derive::Serialize;
use serde_derive::Deserialize;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct Incidence {
    pub timestamp: String,
    pub new_cases: u32,
}

#[derive(Serialize, Deserialize)]
pub struct Death {
    pub timestamp: String,
    pub deaths: u32,
}

create_report_trait!(Incidence);
create_report_trait!(Death);

fn main() {
    let global_state = Arc::new(Mutex::new(GlobalState::new()));
    
    let mut context1 = Context::new(&global_state);
    let mut context2 = Context::new(&global_state);

    context1.add_report::<Incidence>("incidence_report.csv");
    context1.add_report::<Death>("death_report.csv");

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