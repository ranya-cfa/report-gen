use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::any::TypeId;
use std::sync::mpsc::Sender;
use crate::GlobalState;
use crate::Report;
use std::fs::File;
use std::io::{BufRead, BufReader, Result};

pub struct Context {
    global_state: Arc<Mutex<GlobalState>>,
    report_senders: HashMap<TypeId, Sender<Box<dyn Report + Send>>>, // Type Id represents type of report, value is Sender 
}

impl Context {
    pub fn new(global_state: Arc<Mutex<GlobalState>>) -> Self {
        Context {
            global_state,
            report_senders: HashMap::new(),
        }
    }

    pub fn release_report_item<T: Report + 'static>(&mut self, item: T) { // Route report item to appropriate channel 
        // Releases a report item to the corresponding channel.
        let type_id = TypeId::of::<T>();
        //Check if sender is known
        if !self.report_senders.contains_key(&type_id) {
            let global_state = self.global_state.lock().unwrap();
            if let Some(sender) = global_state.get_report_sender::<T>() {
                self.report_senders.insert(type_id, sender.clone()); //needs clone for type matching
            } else {
                println!("No sender found for the report type.");
                return;
            }
        }
        if let Some(sender) = self.report_senders.get(&type_id) {
            sender.send(Box::new(item)).unwrap();
        } else {
            eprintln!("Error: No sender found for report type {:?}", type_id);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Incidence, Death};
    use std::sync::Arc;
    use std::sync::Mutex;

    #[test]
    fn test_context_creation() {
        let global_state = Arc::new(Mutex::new(GlobalState::new()));
        let context = Context::new(global_state);
        assert!(context.report_senders.is_empty()); // Ensure that no reports have been added yet
    }

    #[test]
    fn test_release_report_item() {
        let mut state = GlobalState::new();
        state.setup_report::<Incidence>("test3_incidence_report.csv");
        state.setup_report::<Death>("test3_death_report.csv");
    
        // generate reports
        let incidence_report = Incidence {
            timestamp: "2023-06-26 0".to_string(),
            new_cases: 150,
        };
    
        let death_report = Death {
            timestamp: "2023-06-26 0".to_string(),
            deaths: 5,
        };
    
        // send reports
        if let Some(sender) = state.get_report_sender::<Incidence>() {
            sender.send(Box::new(incidence_report.clone())).unwrap();
        }
    
        if let Some(sender) = state.get_report_sender::<Death>() {
            sender.send(Box::new(death_report.clone())).unwrap();
        }
    
        state.join_threads();
    
        // verify contents
        assert!(std::path::Path::new("test3_incidence_report.csv").exists(), "Incidence report file does not exist");
        assert!(std::path::Path::new("test3_death_report.csv").exists(), "Death report file does not exist");
    
        verify_file::<Incidence>("test3_incidence_report.csv", vec![
            "timestamp,new_cases",
            "2023-06-26 0,150"
        ]);
    
        verify_file::<Death>("test3_death_report.csv", vec![
            "timestamp,deaths",
            "2023-06-26 0,5"
        ]);
    
        std::fs::remove_file("test3_incidence_report.csv").unwrap();
        std::fs::remove_file("test3_death_report.csv").unwrap();
    }
    
}

fn verify_file<T: Report>(file_path: &str, expected_lines: Vec<&str>) {
    let file = File::open(file_path).expect("Failed to open file");
    let reader = BufReader::new(file);
    
    // read file contents into vector of strings 
    let mut lines: Vec<String> = reader.lines().map(|line| line.expect("Failed to read line")).collect();

    lines.retain(|line| !line.trim().is_empty()); // remove extra empty lines at end of file

    assert_eq!(lines.len(), expected_lines.len(), "File line count does not match");

    // compare each line
    for (line, expected_line) in lines.iter().zip(expected_lines.iter()) {
        assert_eq!(line, *expected_line, "File contents don't match");
    }
 }
