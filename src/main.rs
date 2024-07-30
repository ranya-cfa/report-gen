mod global_state;
mod context;

use global_state::GlobalState;
use context::Context;
use std::sync::{Arc, Mutex};
use csv::Writer;
use std::fs::File;
use serde_derive::{Serialize, Deserialize};

pub trait Report: Send + 'static {
    fn make_report(&self);
    fn serialize(&self, writer: &mut Writer<File>);
}

#[derive(Serialize, Deserialize)]
#[derive(Clone)]
pub struct Incidence {
    pub timestamp: String,
    pub new_cases: u32,
}

#[derive(Serialize, Deserialize)]
#[derive(Clone)]
pub struct Death {
    pub timestamp: String,
    pub deaths: u32,
}

macro_rules! create_report_trait {
    ($name:ident) => {
        impl Report for $name {
            fn make_report(&self) {
                println!("{} Report", stringify!($name));
            }

            fn serialize(&self, writer: &mut Writer<File>) {
                writer.serialize(self).unwrap();
            }
        }
    };
}

create_report_trait!(Incidence);
create_report_trait!(Death);

fn main() {
    let global_state = Arc::new(Mutex::new(GlobalState::new()));
    
    let mut context1 = Context::new(global_state.clone());
    let mut context2 = Context::new(global_state.clone());

    global_state.lock().unwrap().add_report::<Incidence>("incidence_report.csv");
    global_state.lock().unwrap().add_report::<Death>("death_report.csv");

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

    // Ensure all threads are joined
    global_state.lock().unwrap().join_threads();
}
