use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::any::TypeId;
use std::sync::mpsc::Sender;
use crate::GlobalState;
use crate::Report;

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
}