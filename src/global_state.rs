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
pub trait Report: Send + 'static { // Send is necessary to ensure thread safety because we have multiple thread boundaries with 'Sender' and 'Receiver'
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
        self.report_senders.insert(TypeId::of::<T>(), tx); // Insert sender into report_senders map. Key: Type identifier of report type 'T'
        let filename = filename.to_string();
        thread::spawn(move || { // Spawn new thread to process incoming reports
            let file = File::create(&filename).unwrap(); 
            let mut writer = Writer::from_writer(file); // Create writer for that specific file 
            println!("Started processing reports for {}", filename);
            loop {
                match rx.recv_timeout(Duration::from_secs(2)) { // Receive report from receiver (Wait 2 seconds before timing out)
                    Ok(received) => {
                        received.make_report(); // Create report 
                        received.serialize(&mut writer); // Serialize into csv using designated writer
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
    pub fn get_report_sender<T: Report + 'static>(&self) -> Option<&Sender<Box<dyn Report + Send>>> { // Return the Sender
        self.report_senders.get(&TypeId::of::<T>())
    }

    pub fn get_report_map(&self) -> HashMap<TypeId, Sender<Box<dyn Report + Send>>> {
        self.report_senders.clone()
    }
}

lazy_static! { // create globally accessible, thread safe instance of Global Instance that can be shared across multiple threads 
    pub static ref GLOBAL_STATE: Arc<Mutex<GlobalState>> = Arc::new(Mutex::new(GlobalState::new()));//creates new instance of GlobalState, mutex ensures that access is synchronized, arc allows to be shared across threads 
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