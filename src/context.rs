use crate::GlobalState;
use crate::Report;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};

pub struct Context {
    sender: Sender<Box<dyn Report>>,
}

impl Context {
    pub fn new(global_state: Arc<Mutex<GlobalState>>) -> Self {
        let sender = global_state.lock().unwrap().get_sender();
        Context {
            sender,
        }
    }

    pub fn send_report<T: Report>(&self, report: T) {
        self.sender.send(Box::new(report)).unwrap(); // sends report through channel
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
        assert_eq!(
            global_state
                .lock()
                .unwrap()
                .get_receiver()
                .lock()
                .unwrap()
                .try_recv()
                .err()
                .unwrap(),
            std::sync::mpsc::TryRecvError::Empty
        ); // Ensure that no reports have been added yet
    }

    #[test]
    fn test_send_report() {
        let global_state = Arc::new(Mutex::new(GlobalState::new()));

        {
            let mut state = global_state.lock().unwrap();
            state.register_report_type::<Incidence>("test3_incidence_report.csv");
            state.register_report_type::<Death>("test3_death_report.csv");
        }

        let context_name = "Context 1".to_string();

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

        {
            let mut state = global_state.lock().unwrap();
            state.start_consumer_thread();
        }

        {
            let context = Context::new(global_state.clone());
            context.send_report(incidence_report);
            context.send_report(death_report);
        }

        global_state.lock().unwrap().join_consumer_thread();
        drop(global_state);

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
        let num_contexts = 4;
        let global_state = Arc::new(Mutex::new(GlobalState::new()));

        {
            let mut state = global_state.lock().unwrap();
            state.register_report_type::<Incidence>("test4_incidence_report.csv");
            state.register_report_type::<Death>("test4_death_report.csv");
            state.start_consumer_thread();
        }

        let mut handles = vec![];

        for i in 0..num_contexts {
            let global_state_clone = global_state.clone();
            let handle = thread::spawn(move || {
                let context = Context::new(global_state_clone);
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
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        global_state.lock().unwrap().join_consumer_thread();
        drop(global_state);

        assert!(
            std::path::Path::new("test4_incidence_report.csv").exists(),
            "Incidence report file does not exist"
        );
        assert!(
            std::path::Path::new("test4_death_report.csv").exists(),
            "Death report file does not exist"
        );

        verify_file_multi::<Incidence>(
            "test4_incidence_report.csv",
            "context_name,counter,timestamp,new_cases",
        );
        verify_file_multi::<Death>(
            "test4_death_report.csv",
            "context_name,counter,timestamp,deaths",
        );

        std::fs::remove_file("test4_incidence_report.csv").unwrap();
        std::fs::remove_file("test4_death_report.csv").unwrap();
    }
}

fn verify_single_thread<T: Report>(file_path: &str, expected_lines: Vec<&str>) {
    let file = File::open(file_path).expect("Failed to open file");
    let reader = BufReader::new(file);

    let mut lines: Vec<String> = reader
        .lines()
        .map(|line| line.expect("Failed to read line"))
        .collect();

    println!("Actual file contents:");
    for line in &lines {
        println!("Line: {}", line);
    }

    lines.retain(|line| !line.trim().is_empty());
    lines.sort();

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

    for (line, expected_line) in lines.iter().zip(expected_lines_sorted.iter()) {
        assert_eq!(line, expected_line, "File contents don't match");
    }
}

fn verify_file_multi<T: Report>(file_path: &str, expected_header: &str) {
    let file = File::open(file_path).expect("Failed to open file");
    let reader = BufReader::new(file);

    // read file contents into vec of strings
    let lines: Vec<String> = reader
        .lines()
        .map(|line| line.expect("Failed to read line"))
        .collect();

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
        assert_eq!(
            counter, *expected_counter,
            "Counter value does not match for context {}",
            context_name
        );

        *expected_counter += 1;
    }
}
