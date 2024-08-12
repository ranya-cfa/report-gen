mod context;

use context::Context;
use csv::Writer;
use serde_derive::{Deserialize, Serialize};
use std::any::TypeId;
use std::fs::File;
use std::thread;

pub trait Report: 'static {
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
    pub counter: usize,
    pub timestamp: String,
    pub new_cases: u32,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Death {
    pub counter: usize,
    pub timestamp: String,
    pub deaths: u32,
}

create_report_trait!(Incidence);
create_report_trait!(Death);

fn main() {
    let context_names = vec!["context_0", "context_1", "context_2", "context_3"]; // user must input context names here 
    let mut handles = vec![];

    for context_name in context_names {
        let context_name = context_name.to_string();
        let handle = thread::spawn(move || {
            let mut context = Context::new(context_name.clone());
            context.add_report::<Incidence>();
            context.add_report::<Death>();
            for counter in 0..4 {
                let incidence_report = Incidence {
                    counter,
                    timestamp: format!("2023-06-26 {}", counter),
                    new_cases: 150 + counter as u32,
                };
                let death_report = Death {
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