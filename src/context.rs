use crate::GlobalState;
use crate::Report;
use std::any::TypeId;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};

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

    pub fn release_report_item<T: Report + 'static>(&mut self, item: T) {
        // Route report item to appropriate channel
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
    use crate::{Death, Incidence};
    use std::sync::Arc;
    use std::sync::Mutex;
    use std::thread;

    #[test]
    fn test_context_creation() {
        let global_state = Arc::new(Mutex::new(GlobalState::new()));
        let context = Context::new(global_state);
        assert!(context.report_senders.is_empty());
    }

    #[test]
    fn test_release_report_item() {
        let mut state = GlobalState::new();
        state.add_report::<Incidence>("test3_incidence_report.csv");
        state.add_report::<Death>("test3_death_report.csv");

        let context_name = "Context 1".to_string();
        // generate reports
        let incidence_report = Incidence {
            context_name: context_name.clone(),
            counter: 0,
            timestamp: "2023-06-26 0".to_string(),
            new_cases: 150,
        };

        let death_report = Death {
            context_name: context_name.clone(),
            counter: 0,
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
        assert!(
            std::path::Path::new("test3_incidence_report.csv").exists(),
            "Incidence report file does not exist"
        );
        assert!(
            std::path::Path::new("test3_death_report.csv").exists(),
            "Death report file does not exist"
        );

        verify_single_thread::<Incidence>(
            "test3_incidence_report.csv",
            vec![
                "context_name,counter,timestamp,new_cases",
                "Context 1,0,2023-06-26 0,150",
            ],
        );

        verify_single_thread::<Death>(
            "test3_death_report.csv",
            vec![
                "context_name,counter,timestamp,deaths",
                "Context 1,0,2023-06-26 0,5",
            ],
        );
        std::fs::remove_file("test3_incidence_report.csv").unwrap();
        std::fs::remove_file("test3_death_report.csv").unwrap();
    }

    #[test]
    fn test_mult_context_prod() {
        let num_contexts = 2;
        let mut global_state = GlobalState::new();

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

        verify_file_multi::<Incidence>("test4_incidence_report.csv", "context_name,counter,timestamp,new_cases");
        verify_file_multi::<Death>("test4_death_report.csv", "context_name,counter,timestamp,deaths");

        std::fs::remove_file("test4_incidence_report.csv").unwrap();
        std::fs::remove_file("test4_death_report.csv").unwrap();
    }
}

fn verify_single_thread<T: Report>(file_path: &str, expected_lines: Vec<&str>) {
    let file = File::open(file_path).expect("Failed to open file");
    let reader = BufReader::new(file);

    // read file contents into vector of strings
    let mut lines: Vec<String> = reader
        .lines()
        .map(|line| line.expect("Failed to read line"))
        .collect();

    // Debug prints to show file contents
    println!("Actual file contents:");
    for line in &lines {
        println!("Line: {}", line);
    }

    lines.retain(|line| !line.trim().is_empty()); // remove extra empty lines at end of file
    lines.sort(); // Sort lines for comparison

    // Sort expected lines for comparison
    let mut expected_lines_sorted = expected_lines
        .into_iter()
        .map(|line| line.to_string())
        .collect::<Vec<_>>();
    expected_lines_sorted.sort();

    assert_eq!(
        lines.len(),
        expected_lines_sorted.len(),
        "File line count does not match"
    );

    // compare each line
    for (line, expected_line) in lines.iter().zip(expected_lines_sorted.iter()) {
        assert_eq!(line, expected_line, "File contents don't match");
    }
}

fn verify_file_multi<T: Report>(file_path: &str, expected_header: &str) {
    let file = File::open(file_path).expect("Failed to open file");
    let reader = BufReader::new(file);

    // read file contents into vec of strings
    let lines: Vec<String> = reader.lines().map(|line| line.expect("Failed to read line")).collect();

    println!("Actual file contents:");
    for line in &lines {
        println!("Line: {}", line);
    }

    // first line should be the header
    assert_eq!(lines[0], expected_header, "File header does not match");

    let mut context_counters: HashMap<String, i32> = HashMap::new();

    // check if counters are in correct order
    for line in lines.iter().skip(1) {
        let parts: Vec<&str> = line.split(',').collect();
        let context_name = parts[0].to_string();
        let counter: i32 = parts[1].parse().expect("Failed to parse counter");

        let expected_counter = context_counters.entry(context_name.clone()).or_insert(0);
        assert_eq!(counter, *expected_counter, "Counter value does not match for context {}", context_name);

        *expected_counter += 1;
    }
}