use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::any::TypeId;
use std::sync::mpsc::{Sender, Receiver};
use serde::{Serialize, Deserialize};
use crate::GlobalState;

pub struct Context<'gs> {
    global_state: &'gs Arc<Mutex<GlobalState>>, // Lifetime 'gs ensures that reference to global state outlives 'Context' struct 
    report_senders: HashMap<TypeId, Sender<Box<dyn Report + Send>>>, // Type Id represents type of report, value is Sender 
}

impl<'gs> Context<'gs> {
    pub fn new(global_state: &'gs Arc<Mutex<GlobalState>>) -> Self {
        Context {
            global_state,
            report_senders: HashMap::new(),
        }
    }

    pub fn release_report_item<T: Report + 'static>(&self, item: T) { // Route report item to appropriate channel 
        // Releases a report item to the corresponding channel.
        let type_id = TypeId::of::<T>();
        //Check if sender is known
        let sender = if let Some(sender) = self.report_senders.get(&type_id) {
            sender.clone() // Clone the sender to get an owned copy
        } else {
            // Lock global state and retrieve sender if it is not currently known 
            let s = {
                let global_state = self.global_state.lock().unwrap();
                global_state.get_report_sender::<T>().cloned()
            };

            if let Some(s) = s { // if the sender is found, store it in local map 
                self.report_senders.insert(type_id, s.clone());
                s
            } else {
                println!("No sender found for the report type.");
                return;
            }
        };

        // Send the report item using the sender
        if let Err(_) = sender.send(Box::new(item)) {
            println!("Failed to send report item.");
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

    /* #[test]
    fn test_add_report() {
        let global_state = Arc::new(Mutex::new(GlobalState::new()));
        let mut context = Context::new(&global_state);
        context.add_report::<Incidence>("incidence_report.csv");
        {
            let state = global_state.lock().unwrap();
            assert!(state.get_report_sender::<Incidence>().is_some());
        }
        let mut state = global_state.lock().unwrap();
        state.join_threads();
    } */
}