mod context;
mod global_state;

use context::Context;
use csv::Writer;
use global_state::GlobalState;
use serde_derive::{Deserialize, Serialize};
use std::fs::File;
use std::sync::{Arc, Mutex};
use std::thread;

pub trait Report: Send + 'static {
    fn serialize(&self, writer: &mut Writer<File>);
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

macro_rules! create_report_trait {
    ($name:ident) => {
        impl Report for $name {
            fn serialize(&self, writer: &mut Writer<File>) {
                writer.serialize(self).unwrap();
            }
        }
    };
}

create_report_trait!(Incidence);
create_report_trait!(Death);

fn main() {
    let num_contexts = 2;
    let let mut global_state = GlobalState::new();

    global_state.add_report::<Incidence>("test4_incidence_report.csv");
    global_state.add_report::<Death>("test4_death_report.csv");

    let global_state = Arc::new(Mutex::new(global_state));

    let mut handles = vec![];

    for i in 0..num_contexts {
        let global_state = Arc::clone(&global_state);
        let handle = thread::spawn(move || {
            let mut context = Context::new(global_state);
            for counter in 0..3 {
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
                context.release_report_item(incidence_report);
                context.release_report_item(death_report);
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    global_state.lock().unwrap().join_threads();
}