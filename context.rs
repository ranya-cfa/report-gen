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

pub struct Context {
    //plan_queue: PlanQueue,
    //callback_queue: VecDeque<Box<Callback>>,
    //plugin_data: HashMap<TypeId, Box<dyn Any>>,
    //time: f64,
    report_map: HashMap<TypeId, Box<dyn Any>>, //hashmap of report senders
}

impl Context {
    pub fn new() -> Context {
        Context {
            //plan_queue: PlanQueue::new(),
            //callback_queue: VecDeque::new(),
            //plugin_data: HashMap::new(),
            //time: 0.0,
            //report_map: HashMap::new(),
        }
    }
    
    pub fn add_report<T: Report + 'static>(&mut self, filename: &str) {
        // Registers a new report type and starts a consumer thread to process reports of this type.
        let (tx, rx): (Sender<T>, Receiver<T>) = mpsc::channel();
        thread::spawn(move || {
            process_reports(rx, &filename);
        });
        self.report_senders.insert(TypeId::of::<T>(), Box::new(tx));
    }

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

    pub fn release_report_item<T: Report + 'static>(&self, item: T) {
        // Releases a report item to the corresponding channel.
        if let Some(sender) = self.report_senders.get(&TypeId::of::<T>()) {
            let sender = sender.downcast_ref::<Sender<T>>().unwrap();
            sender.send(item).unwrap();
        }
    }
}




//main.rs
fn main() {
    let mut context = Context::new();

    context.add_report::<Incidence>("incidence_report.csv");
    context.add_report::<Death>("death_report.csv");

    context.release_report_item(Incidence {
        timestamp: "2023-06-26 0".to_string(),
        new_cases: 150,
    });
    context.release_report_item(Death {
        timestamp: "2023-06-26 0".to_string(),
        deaths: 5,
    });

    context.execute();
}