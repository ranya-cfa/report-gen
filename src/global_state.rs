use crate::Report;
use csv::Writer;
use std::any::TypeId;
use std::collections::HashMap;
use std::fs::File;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};

pub struct GlobalState {
    sender: Option<Sender<Box<dyn Report>>>, // Option allows sender to be set to None after consumer has been joined. Indicates that no more reports can be sent. 
    receiver: Arc<Mutex<Receiver<Box<dyn Report>>>>,
    typeid_to_writer: Arc<Mutex<HashMap<TypeId, Writer<File>>>>,
    consumer_thread: Option<JoinHandle<()>>,
}

impl GlobalState {
    pub fn new() -> Self {
        let (sender, receiver): (Sender<Box<dyn Report>>, Receiver<Box<dyn Report>>) =
            mpsc::channel();
        GlobalState {
            sender: Some(sender),                                                 // sender for sending reports
            receiver: Arc::new(Mutex::new(receiver)), // receiver, wrapped in Arc and Mutex for thread safety
            typeid_to_writer: Arc::new(Mutex::new(HashMap::new())), // maps type IDs to CSV writers, arc because it needs to be shared across thread
            consumer_thread: None,                                  // handle for consumer thread
        }
    }

    pub fn register_report_type<T: Report>(&mut self, filename: &str) {
        let file = File::create(filename).unwrap(); // create new file for report type
        println!("{} file created", filename); // print that the file was created
        self.typeid_to_writer
            .lock()
            .unwrap()
            .insert(TypeId::of::<T>(), Writer::from_writer(file)); // insert writer for report type into map
    }

    pub fn start_consumer_thread(&mut self) {
        let receiver = Arc::clone(&self.receiver); // arc must be cloned
        let typeid_to_writer = Arc::clone(&self.typeid_to_writer);
        let handle = thread::spawn(move || {
            // spawn new thread to process incoming reports
            loop {
                let result = receiver.lock().unwrap().recv(); // receive a report
                match result {
                    // Receive report from receiver (Wait 2 seconds before timing out)
                    Ok(received) => {
                        received.make_report(); // create report
                        let type_id = received.type_id();
                        let mut writers = typeid_to_writer.lock().unwrap();
                        if let Some(writer) = writers.get_mut(&type_id) {
                            received.serialize(writer); // serialize report with appropriate writer
                            println!("Written report to file");
                        } else {
                            eprintln!("No writer found for report type");
                        }
                    }
                    Err(_) => {
                        println!("No more reports to process");
                        break;
                    }
                }
            }
        });
        self.consumer_thread = Some(handle); // store handle to consumer thread
    }

    pub fn join_consumer_thread(&mut self) {
        if let Some(handle) = self.consumer_thread.take() {
            handle.join().unwrap();
        }
    }

    pub fn get_sender(&self) -> Sender<Box<dyn Report>> {
        self.sender.as_ref().unwrap().clone()
    }

    pub fn get_receiver(&self) -> Arc<Mutex<Receiver<Box<dyn Report>>>> {
        Arc::clone(&self.receiver)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Death, Incidence};
    use std::fs;

    #[test]
    fn test_global_state_creation() {
        let state = GlobalState::new();
        assert!(state.consumer_thread.is_none());
        assert_eq!(
            state.receiver.lock().unwrap().try_recv().err().unwrap(),
            std::sync::mpsc::TryRecvError::Empty
        );
    }

    #[test]
    fn test_setup_inc_report() {
        let mut state = GlobalState::new();
        state.register_report_type::<Incidence>("test_incidence_report.csv");
        assert!(state
            .typeid_to_writer
            .lock()
            .unwrap()
            .contains_key(&TypeId::of::<Incidence>()));
        std::fs::remove_file("test_incidence_report.csv").unwrap();
    }

    #[test]
    fn test_setup_death_report() {
        let mut state = GlobalState::new();
        state.register_report_type::<Death>("test_death_report.csv");
        assert!(state
            .typeid_to_writer
            .lock()
            .unwrap()
            .contains_key(&TypeId::of::<Death>()));
        std::fs::remove_file("test_death_report.csv").unwrap();
    }

    #[test]
    fn test_start_consumer_thread() {
        let mut global_state = GlobalState::new();
        global_state.register_report_type::<Death>("test_death2_report.csv");
        global_state.start_consumer_thread();
        assert!(global_state.consumer_thread.is_some());
        fs::remove_file("test_death2_report.csv").unwrap();
    }
}
