mod context;
mod global_state;

use context::Context;
use csv::Writer;
use global_state::GlobalState;
use serde_derive::{Deserialize, Serialize};
use std::any::TypeId;
use std::fs::File;
use std::sync::{Arc, Mutex};

pub trait Report: Send + 'static {
    // Send is necessary to ensure thread safety because we have multiple thread boundaries with 'Sender' and 'Receiver'
    fn make_report(&self);
    fn type_id(&self) -> TypeId;
    fn serialize(&self, writer: &mut Writer<File>);
}

macro_rules! create_report_trait {
    ($name:ident) => {
        impl Report for $name {
            fn make_report(&self) {
                println!("{} Report", stringify!($name));
            }

            fn type_id(&self) -> TypeId {
                // returns the TypeId of the report (used for identification)
                TypeId::of::<$name>()
            }

            fn serialize(&self, writer: &mut Writer<File>) {
                writer.serialize(self).unwrap();
            }
        }
    };
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Incidence {
    pub context_name: String,
    pub counter: usize,
    pub timestamp: String,
    pub new_cases: u32,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Death {
    pub context_name: String,
    pub counter: usize,
    pub timestamp: String,
    pub deaths: u32,
}

create_report_trait!(Incidence);
create_report_trait!(Death);

fn main() {
    let global_state = Arc::new(Mutex::new(GlobalState::new()));

    let context1 = Context::new(global_state.clone());
    let context2 = Context::new(global_state.clone());

    global_state
        .lock()
        .unwrap()
        .register_report_type::<Incidence>("incidence.csv");

    global_state
        .lock()
        .unwrap()
        .register_report_type::<Death>("death.csv");

    context1.send_report(Incidence {
        context_name: "Context 2".to_string(),
        counter: 1,
        timestamp: "2024-07-26".to_string(),
        new_cases: 42,
    });
    context2.send_report(Death {
        context_name: "Context 2".to_string(),
        counter: 1,
        timestamp: "2024-07-26".to_string(),
        deaths: 1,
    });

    global_state.lock().unwrap().join_consumer_thread();
}
