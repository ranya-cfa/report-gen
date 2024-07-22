use std::sync::{Arc, Mutex};
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::any::TypeId;
use serde_derive::Serialize;
use serde_derive::Deserialize;
use std::sync::mpsc;
use std::sync::mpsc::{Sender, Receiver};
use std::fs::File;
use std::thread;
use std::time::Duration;
use serde::{Serialize, Deserialize};
use csv::Writer;
pub trait Report: Send + 'static {
    fn make_report(&self);
    fn serialize(&self, writer: &mut Writer<File>);
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

#[derive(Serialize, Deserialize)]
pub struct Incidence {
    timestamp: String,
    new_cases: u32,
}

#[derive(Serialize, Deserialize)]
pub struct Death {
    timestamp: String,
    deaths: u32,
}

create_report_trait!(Incidence);
create_report_trait!(Death);

pub struct GlobalState {
    report_senders: HashMap<TypeId, Sender<Box<dyn Report + Send>>>,
}

impl GlobalState {
    pub fn new() -> Self {
        let mut state = GlobalState {
            report_senders: HashMap::new(),
        };

        state.setup_report::<Incidence>("incidence_report.csv");
        state.setup_report::<Death>("death_report.csv");

        state
    }
    
    // Processes report items from associated receiver channel. 
    pub fn setup_report<T: Report +'static>(&mut self, filename: &str) {
        let (tx, rx): (Sender<Box<dyn Report + Send>>, Receiver<Box<dyn Report + Send>>) = mpsc::channel();
        self.report_senders.insert(TypeId::of::<T>(), tx);
        thread::spawn(move || {
            let file = File::create(filename).unwrap();
            let mut writer = Writer::from_writer(file);
            println!("Started processing reports for {}", filename);
            loop {
                match rx.recv_timeout(Duration::from_secs(2)) {
                    Ok(received) => {
                        received.make_report();
                        received.serialize(&mut writer);
                        println!("Written report to {}", filename);
                    }
                    Err(_) => {
                        println!("No more reports to process for {}", filename);
                        break;
                    }
                }
            }
        });
    }
    // Returns the sender if it exists 
    pub fn get_report_sender<T: Report + 'static>(&self) -> Option<&Sender<Box<dyn Report + Send>>> {
        self.report_senders.get(&TypeId::of::<T>())
    }
}

lazy_static! {
    pub static ref GLOBAL_STATE: Arc<Mutex<GlobalState>> = Arc::new(Mutex::new(GlobalState::new()));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_global_state_creation() {
        let state = GlobalState::new();
        assert!(state.report_senders.is_empty()); // Check that no reports have been added yet
    }

    #[test]
    fn test_setup_report() {
        let mut state = GlobalState::new();
        state.setup_report::<Incidence>("incidence_report.csv");
        assert!(state.get_report_sender::<Incidence>().is_some());

    }
}