use crate::GlobalState;
use crate::Report;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};

pub struct Context {
    global_state: Arc<Mutex<GlobalState>>,
    sender: Sender<Box<dyn Report>>,
}

impl Context {
    pub fn new(global_state: Arc<Mutex<GlobalState>>) -> Self {
        let sender = global_state.lock().unwrap().get_sender();
        Context {
            global_state,
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
    use std::time::Duration;

    #[test]
    fn test_context_creation() {
        let global_state = Arc::new(Mutex::new(GlobalState::new()));
        let context = Context::new(global_state.clone());
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

        let context = Context::new(global_state.clone());

        // consumer thread
        {
            let mut state = global_state.lock().unwrap();
            state.start_consumer_thread();
            context.send_report(incidence_report);
            context.send_report(death_report);
            thread::sleep(Duration::from_secs(3));
            state.join_consumer_thread();
        }

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
