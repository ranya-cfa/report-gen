use std::collections::HashMap;
use std::any::TypeId;
use std::sync::mpsc;
use std::sync::mpsc::{Sender, Receiver};
use std::fs::File;
use std::thread::{self, JoinHandle};
use csv::Writer;
use crate::Report;
pub struct GlobalState {
    report_senders: HashMap<TypeId, Sender<Box<dyn Report + Send>>>,
    threads: Vec<JoinHandle<()>>,
}

impl GlobalState {
    pub fn new() -> Self {
        GlobalState {
            report_senders: HashMap::new(),
            threads: Vec::new(),
        }
    }
    
    // Processes report items from associated receiver channel. 
    pub fn setup_report<T: Report +'static>(&mut self, filename: &str) {
        let (tx, rx): (Sender<Box<dyn Report + Send>>, Receiver<Box<dyn Report + Send>>) = mpsc::channel();
        self.report_senders.insert(TypeId::of::<T>(), tx); // Insert sender into report_senders map. Key: Type identifier of report type 'T'
        let filename = filename.to_string();
        let handle = thread::spawn(move || { // Spawn new thread to process incoming reports
            let file = File::create(&filename).unwrap(); 
            let mut writer = Writer::from_writer(file); // Create writer for that specific file 
            println!("Started processing reports for {}", filename);
            loop {
                match rx.recv() { // Receive report from receiver 
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
        self.threads.push(handle);
    }

    // Returns the sender if it exists 
    pub fn get_report_sender<T: Report + 'static>(&self) -> Option<&Sender<Box<dyn Report + Send>>> { // Return the Sender
        self.report_senders.get(&TypeId::of::<T>())
    }

    pub fn get_report_map(&self) -> HashMap<TypeId, Sender<Box<dyn Report + Send>>> {
        self.report_senders.clone()
    }

    pub fn report_senders_is_empty(&self) -> bool {
        self.report_senders.is_empty()
    }

    pub fn join_threads(&mut self) {
        self.report_senders.clear();
        let handles = std::mem::take(&mut self.threads); 
        for handle in handles {
            handle.join().unwrap();
    }
}

    pub fn add_report<T: Report + 'static>(&mut self, filename: &str) {
        self.setup_report::<T>(filename);
    }
    
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_global_state_creation() {
        let state = GlobalState::new();
        assert!(state.report_senders_is_empty()); // Check that no reports have been added yet
    }

    /* #[test]
    fn test_setup_report() {
        let mut state = GlobalState::new();
        state.setup_report::<Incidence>("incidence_report.csv");
        assert!(state.get_report_sender::<Incidence>().is_some());

    }*/
}