mod context;

use context::Context;
use csv::Writer;
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
    let mut handles = vec![];

    for i in 0..num_contexts {
        let handle = thread::spawn(move || {
            let mut context = Context::new(format!("Context {}", i));
            let incidence_report_name = format!("{}_{}", i, "incidence_report.csv");
            context.add_report::<Incidence>(&incidence_report_name);
            let death_report_name = format!("{}_{}", i, "death_report.csv");
            context.add_report::<Death>(&death_report_name);
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
                // Each context will have its own file. So at the end of this, we will have 4 death files and 4 incidence files 
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }
}