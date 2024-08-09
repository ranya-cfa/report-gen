use csv::Writer;
use crate::Report;
use std::collections::HashMap;
use std::fs::File;
use std::sync::{Arc, Mutex};
use std::any::TypeId;
use std::path::Path;

pub struct Context {
    name: String,
    file_writers: HashMap<TypeId, Arc<Mutex<Writer<File>>>>, // Type Id represents type of report, value is Sender
}

impl Context {
    pub fn new(name: String) -> Self {
        Context {
            name,
            file_writers: HashMap::new(),
        }
    }

    pub fn add_report<T: Report + 'static>(&mut self, filename: &str) {
        let path = Path::new(filename);
        let file = File::create(path).expect("Couldn't create file");
        let writer = Writer::from_writer(file);
        self.file_writers.insert(TypeId::of::<T>(), Arc::new(Mutex::new(writer)));
    }

    pub fn send_report<T:Report>(&self, report: T) {
        if let Some(writer) = self.file_writers.get(&report.type_id()){
            let mut writer = writer.lock().unwrap();
            report.serialize(&mut *writer);
        } else {
            panic!("No writer found for the report type");
        }
    }
}