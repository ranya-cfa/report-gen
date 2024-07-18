use crate::global_state::{GLOBAL_STATE, Report, Incidence, Death};

pub struct Context {
    // fields
}

impl Context {
    pub fn new() -> Context {
        Context {
            // fields
        }
    }

pub fn add_report<T: Report + 'static>(&mut self, filename: &str) {
    GLOBAL_STATE.lock().unwrap().add_report::<T>(filename);
}

pub fn release_report_item<T: Report + 'static>(&self, item: T) {
    // Releases a report item to the corresponding channel.
    if let Some(sender) = GLOBAL_STATE.lock().get_report_sender::<T>() {
        sender.send(item).unwrap();
    } else {
        println!("No sender found for the report type.");
    }
}
}