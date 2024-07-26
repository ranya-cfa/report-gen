use std::sync::{Arc, Mutex};
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::any::TypeId;
use serde_derive::Serialize;
use serde_derive::Deserialize;
use std::sync::mpsc;
use std::sync::mpsc::{Sender, Receiver};
use std::fs::File;
use std::thread::{self, JoinHandle};
use std::time::Duration;
use serde::{Serialize, Deserialize};
use csv::Writer;
pub trait Report: Send + 'static { // Send is necessary to ensure thread safety because we have multiple thread boundaries with 'Sender' and 'Receiver'
    fn make_report(&self);
    fn type_id(&self) -> TypeId;
    fn serialize(&self, writer: &mut Writer<File>);
}

macro_rules! create_report_trait {
    ($name:ident) => {
        impl Report for $name {
            fn make_report(&self) {
                println!("{} Report", stringify!($name));
            }


            fn type_id(&self) -> TypeId { // returns the TypeId of the report (used for identification)
                TypeId::of::<$name>()
            }

            fn serialize(&self, writer: &mut Writer<File>) {
                writer.serialize(self).unwrap();
            }
        }
    };
}

#[derive(Serialize, Deserialize)]
pub struct Incidence {
    pub timestamp: String,
    pub new_cases: u32,
}

#[derive(Serialize, Deserialize)]
pub struct Death {
    pub timestamp: String,
    pub deaths: u32,
}

create_report_trait!(Incidence);
create_report_trait!(Death);

pub struct GlobalState {
    sender: Sender<Box<dyn Report>>,
    receiver: Arc<Mutex<Receiver<Box<dyn Report>>>>,
    typeid_to_writer: Arc<Mutex<HashMap<TypeId, Writer<File>>>>,
    consumer_thread: Option<JoinHandle<()>>,
}

impl GlobalState {
    pub fn new() -> Self {
        let (sender, receiver): (Sender<Box<dyn Report>>, Receiver<Box<dyn Report>>) = mpsc::channel();
        GlobalState {
            sender, 
            receiver: Arc::new(Mutex::new(receiver)),
            typeid_to_writer: Arc::new(Mutex::new(HashMap::new())),
            consumer_thread: None,
        }
    }

    pub fn register_report_type<T: Report>(&mut self, filename: &str) {
        let file = File::create(filename).unwrap();
        println!("{} file created", filename);
        self.typeid_to_writer.lock().unwrap().insert(TypeId::of::<T>(), Writer::from_writer(file));
    }
    
    pub fn start_consumer_thread(&mut self) {
        let receiver =  Arc::clone(&self.receiver);
        let typeid_to_writer = Arc::clone(&self.typeid_to_writer);
        let handle = thread::spawn(move || { // Spawn new thread to process incoming reports
            loop {
                let result = receiver.lock().unwrap().recv_timeout(Duration::from_secs(2));
                match result{ // Receive report from receiver (Wait 2 seconds before timing out)
                    Ok(received) => {
                        received.make_report(); // Create report
                        let type_id = received.type_id();
                        let mut writers = typeid_to_writer.lock().unwrap();
                        if let Some(writer) = writers.get_mut(&type_id) {
                            received.serialize(writer);
                            println!("Written report to file");
                        } else {
                            eprintln!("No writer found for report type");
                        }
                    }
                    Err(_) => {
                        println!("No more reports to process");
                        break;
                    }
                }
            }
        });

        self.consumer_thread = Some(handle);
    }

    pub fn join_consumer_thread(&mut self) {
        if let Some(handle) = self.consumer_thread.take() {
            handle.join().unwrap();
        }
    }

    pub fn get_sender(&self) -> Sender<Box<dyn Report>> {
        self.sender.clone()
    }
}
