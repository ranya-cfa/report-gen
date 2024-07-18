pub trait Report: Send + 'static {
    fn make_report(&self);
    fn serialize(&self, writer: &mut Writer<File>);
}

macro_rules! create_report_trait {
    ($name:ident) => {
        impl Report for $name {
            fn make_report(&self) {
                println!("{} Report", stringify!($name));
            }

            fn serialize(&self, writer: &mut Writer<File>) {
                writer.serialize(self).unwrap();
            }
        }
    };
}

#[derive(Serialize, Deserialize)]
struct Incidence {
    timestamp: String,
    new_cases: u32,
}

#[derive(Serialize, Deserialize)]
struct Death {
    timestamp: String,
    deaths: u32,
}

create_report_trait!(Incidence);
create_report_trait!(Death);

pub struct GlobalState {
    report_senders: HashMap<TypeId, Box<dyn Any>>
}

impl GlobalState {
    pub fn new() -> Self {
        GlobalState {
            report_senders: HashMap::new()
        }
    }
    // Registers a new report type and starts a consumer thread to process reports of this type.
    pub fn add_report<T: Report + 'static>(&mut self, filename: &str) {
        let (tx, rx): (Sender<T>, Receiver<T>) = mpsc::channel();
        thread::spawn(move || {
            process_reports(rx, &filename);
        });
        self.report_senders.insert(TypeId::of::<T>(), Box::new(tx));
    }
    
    // Processes report items from associated receiver channel. 
    pub fn process_reports<T: Report>(receiver: Receiver<T>, filename: &str) {
        let file = File::create(filename).unwrap();
        let mut writer = Writer::from_writer(file);
        println!("Started processing reports for {}", filename);
    
        loop {
            match receiver.recv_timeout(Duration::from_secs(2)) {
                Ok(received) => {
                    received.make_report();
                    Report::serialize(&received, &mut writer);
                    println!("Written report to {}", filename);
                }
                Err(_) => {
                    println!("No more reports to process for {}", filename);
                    break;
                }
            }
        }
    }
    // Returns the sender if it exists 
    pub fn get_report_sender<T: Report + 'static>(&self) -> Option<&Sender<T>> {
        self.report_senders.get(&TypeId::of::<T>()).and_then(|sender| sender.downcast_ref::<Sender<T>>()) 
    }

    pub fn start_producer_threads(&self) {
        let tx_incidence = self.get_report_sender::<Incidence>().unwrap.clone(); 
        let tx_death = self.get_report_sender::<Death>().unwrap.clone();

        // Producer thread 1 
        let producer1 = thread::spawn(move || {
            for i in 0..3 {
                let timestamp = format!("2023-06-26 {}", i);
                tx_incidence.send(Incidence {timestamp: timestamp.clone(),new_cases: 150 + i}).unwrap();
                tx_death1.send(Death {timestamp, deaths: 5 + i}).unwrap();
                thread::sleep(Duration::from_secs(1));
            }
        });

        // Producer thread 2
        let producer2 = thread::spawn(move || {
            for i in 3..6 {
                let timestamp = format!("2023-06-26 {}", i);
                tx_incidence.send(Incidence {timestamp: timestamp.clone(),new_cases: 150 + i}).unwrap();
                tx_death1.send(Death {timestamp, deaths: 5 + i}).unwrap();
                thread::sleep(Duration::from_secs(1));
            }
        });

        producer1.join().unwrap();
        producer2.join().unwrap();

}
}

    lazy_static::lazy_static! {
        pub static ref GLOBAL_STATE: GlobalState = GlobalState::new();
}