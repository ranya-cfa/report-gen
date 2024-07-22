use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::any::TypeId;
use std::sync::mpsc::{Sender, Receiver};
use serde::{Serialize, Deserialize};
use crate::GlobalState;
use crate::global_state::{GLOBAL_STATE, Report, Incidence, Death};

pub struct Context<'gs> {
    global_state: &'gs Arc<Mutex<GlobalState>>,
    report_senders: HashMap<TypeId, Sender<Box<dyn Report + Send>>>,
}

impl<'gs> Context<'gs> {
    pub fn new(global_state: &'gs Arc<Mutex<GlobalState>>) -> Self {
        let mut context = Context {
            global_state,
            report_senders: HashMap::new(),
        };
        // Initialize the report_senders from the global state
        context.update_report_senders();
        context
    }   

    fn update_report_senders(&mut self) {
        let state = self.global_state.lock().unwrap();
        self.report_senders = state.get_report_sender().clone();
    }

    pub fn add_report<T: Report + 'static>(&mut self, filename: &str) {
        let mut state = self.global_state.lock().unwrap();
        state.setup_report::<T>(filename);
        // After setting up the new report in the global state, update the local hash map
        self.update_report_senders();
    }

    pub fn release_report_item<T: Report + 'static>(&self, item: T) {
        // Releases a report item to the corresponding channel.
        let type_id = TypeId::of::<T>();
        if let Some(sender) = self.report_senders.get(&type_id){
            if let Err(_) = sender.send(Box::new(item)) {
                println!("Failed to send report item.");
            }
        } else {
            println!("No sender found for the report type.");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::sync::Mutex;

    #[test]
    fn test_context_creation() {
        let global_state = Arc::new(Mutex::new(GlobalState::new()));
        let context  = Context::new(&global_state);
        assert!(context.report_senders.is_empty()) // Ensure that no reports have been added yet
    }

    #[test]
    fn test_add_report() {
        let global_state = Arc::new(Mutex::new(GlobalState::new()));
        let context  = Context::new(&global_state);
        context.add_report::<Incidence>("incidence_report.csv");
        let state = global_state.lock().unwrap(); // Check that sender was added
        assert!(state.get_report_senders::<Incidence>().is_some());
    }
}