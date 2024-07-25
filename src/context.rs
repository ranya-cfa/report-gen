use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::any::TypeId;
use std::sync::mpsc::{Sender, Receiver};
use serde::{Serialize, Deserialize};
use crate::GlobalState;
use crate::global_state::{GLOBAL_STATE, Report, Incidence, Death};

pub struct Context<'gs> {
    global_state: &'gs Arc<Mutex<GlobalState>>, // Lifetime 'gs ensures that reference to global state outlives 'Context' struct 
    sender: Sender<Box<dyn Report>>,
}

impl<'gs> Context<'gs> {
    pub fn new(global_state: &'gs Arc<Mutex<GlobalState>>) -> Self {
        let sender = global_state.lock().unwrap().get_sender();
        Context { global_state, sender }
    }

    pub fn add_report<T: Report>(&self, report: T) {
        self.sender.send(Box::new(report)).unwrap();
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
        assert_eq!(global_state.lock().unwrap().receiver.try_recv().err().unwrap(), std::sync::mpsc::TryRecvError::Empty); // Ensure that no reports have been added yet
    }

    #[test]
    fn test_add_report() {
        let global_state = Arc::new(Mutex::new(GlobalState::new()));
        let context = Context::new(&global_state);
        context.add_report(Incidence {
            timestamp: "2023-06-26".to_string(),
            new_cases: 150,
        });
        context.add_report(Death {
            timestamp: "2023-06-26".to_string(),
            deaths: 5,
        });

        let mut state = global_state.lock().unwrap();
        state.join_consumer_thread();
    }
}