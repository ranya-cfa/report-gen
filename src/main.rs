mod context;
mod global_state;

use context::Context;
use csv::Writer;
use global_state::GlobalState;
use serde_derive::{Deserialize, Serialize};
use std::any::TypeId;
use std::fs::File;
use std::sync::{Arc, Mutex};
use std::thread;

pub trait Report: Send + 'static {
    // Send is necessary to ensure thread safety because we have multiple thread boundaries with 'Sender' and 'Receiver'
    fn type_id(&self) -> TypeId;
    fn serialize(&self, writer: &mut Writer<File>);
}

macro_rules! create_report_trait {
    ($name:ident) => {
        impl Report for $name {
            
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
    let num_contexts = 4;
    let global_state = Arc::new(Mutex::new(GlobalState::new()));

    {
        let mut state = global_state.lock().unwrap();
        state.register_report_type::<Incidence>("incidence_report.csv");
        state.register_report_type::<Death>("death_report.csv");
    }

    {
        let mut state = global_state.lock().unwrap();
        state.start_consumer_thread();
    }

    let mut handles = vec![];

    for i in 0..num_contexts {
        let global_state_clone = global_state.clone();
        let handle = thread::spawn(move || {
            let context = Context::new(global_state.clone());
            for counter in 0..4 {
                let incidence_report = Incidence {
                    context_name: format!("Context {}", i),
                    counter,
                    timestamp: format!("2023-06-26 {}", counter),
                    new_cases: 150 + counter as u32,
                };
                let death_report = Death {
                    context_name: format!("Context {}", i),
                    counter,
                    timestamp: format!("2023-06-26 {}", counter),
                    deaths: 5 + counter as u32,
                };
                context.send_report(incidence_report);
                context.send_report(death_report);
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    global_state.lock().unwrap().join_consumer_thread();
}
