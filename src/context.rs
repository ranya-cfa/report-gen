use std::sync::{Arc, Mutex};
use std::sync::mpsc::Sender;
use crate::GlobalState;
use crate::Report;

pub struct Context{
    global_state: Arc<Mutex<GlobalState>>,
    sender: Sender<Box<dyn Report>>,
}

impl Context {
    pub fn new(global_state: Arc<Mutex<GlobalState>>) -> Self {
        let sender = global_state.lock().unwrap().get_sender();
        Context { global_state, sender }
    }

    pub fn send_report<T: Report>(&self, report: T) {
        self.sender.send(Box::new(report)).unwrap(); // sends report through channel
    }
}
