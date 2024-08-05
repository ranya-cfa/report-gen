use crate::Report;
use csv::Writer;
use std::any::TypeId;
use std::collections::HashMap;
use std::fs::File;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::thread::{self, JoinHandle};
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
    pub fn setup_report<T: Report + 'static>(&mut self, filename: &str) {
        let (tx, rx): (
            Sender<Box<dyn Report + Send>>,
            Receiver<Box<dyn Report + Send>>,
        ) = mpsc::channel();
        self.report_senders.insert(TypeId::of::<T>(), tx); // Insert sender into report_senders map. Key: Type identifier of report type 'T'
        let filename = filename.to_string();
        let handle = thread::spawn(move || {
            // Spawn new thread to process incoming reports
            let file = File::create(&filename).unwrap();
            let mut writer = Writer::from_writer(file); // Create writer for that specific file
            loop {
                match rx.recv() {
                    // Receive report from receiver
                    Ok(received) => {
\                        received.serialize(&mut writer); // Serialize into csv using designated writer
\                    }
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
    pub fn get_report_sender<T: Report + 'static>(
        &self,
    ) -> Option<&Sender<Box<dyn Report + Send>>> {
        // Return the Sender
        self.report_senders.get(&TypeId::of::<T>())
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
    use crate::{Death, Incidence};

    #[test]
    fn test_global_state_creation() {
        let state = GlobalState::new();
        assert!(state.report_senders_is_empty()); // Check that no reports have been added yet
    }

    #[test]
    fn test_setup_inc_report() {
        let mut state = GlobalState::new();
        state.setup_report::<Incidence>("test_incidence_report.csv");
        assert!(state.get_report_sender::<Incidence>().is_some());
        state.report_senders.clear();
        state.join_threads();
        assert!(
            std::path::Path::new("test_incidence_report.csv").exists(),
            "Incidence report file should exist"
        );
        std::fs::remove_file("test_incidence_report.csv").unwrap();
    }

    #[test]
    fn test_setup_death_report() {
        let mut state = GlobalState::new();
        state.setup_report::<Death>("test_death_report.csv");
        assert!(state.get_report_sender::<Death>().is_some());
        state.report_senders.clear();
        state.join_threads();
        assert!(
            std::path::Path::new("test_death_report.csv").exists(),
            "Death report file should exist"
        );
        std::fs::remove_file("test_death_report.csv").unwrap();
    }

    #[test]
    fn test_join_threads() {
        let mut state = GlobalState::new();
        state.setup_report::<Incidence>("test2_incidence_report.csv");
        state.setup_report::<Death>("test2_death_report.csv");
        assert!(!state.threads.is_empty());
        state.report_senders.clear();
        state.join_threads();
        assert!(state.threads.is_empty());
        std::fs::remove_file("test2_incidence_report.csv").unwrap();
        std::fs::remove_file("test2_death_report.csv").unwrap();
    }
}
