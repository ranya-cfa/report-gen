# Option 2

## Overview 

One channel, with all report types written onto it and one reader thread that demuxes them.

## Global State Struct

The GlobalState struct manages the state and coordination of report generation in the simulation. It provides functionality to register new report types, manages channels, and handles consumer threads. 

### Fields

#### `sender: Option<Sender<Box<dyn Report>>>`
Optional Sender that can send Box<dyn Report> objects. Option allows Sender to be set to None after the consumer thread has been joined, which indicates that no more reports can be sent.

#### `receiver: Arc<Mutex<Receiver<Box<dyn Report>>>>`
Channel receiver, used to safely receive reports from different threads. 

#### `typeid_to_writer: Arc<Mutex<HashMap<TypeId, Writer<File>>>>`
A map that associates `TypeId` with a `csv::Writer<File> objects`. This allows the system to write different types of reports to different files based on their type ID.

- **Key**: `TypeId` - The unique identifier for the type of report.
- **Value**: `Writer<File>` - CSV writer object associated with a file. Each TypeId maps to a specific `Writer<File>`, allowing different types of reports to be written to different files.

#### `consumer_thread: Option<JoinHandle<()>>`
Optional thread handle for the consumer thread. 

### Methods 

#### `new() -> Self`

Creates and returns a new GlobalState instance with empty fields.

#### `register_report_type<T: Report>(&mut self, filename: &str)`

Creates a new file for the report type and inserts a writer for that report type into the typeid_to_writer hashmap.  

`T` - The type of the report to be registered.
`filename` - The file path where the report of this type will be saved.

#### `join_consumer_thread(&mut self)`

`self.sender = None` is necessary to tell the receiver that no new report will ever be sent. Then, join the consumer thread. 

#### `get_sender(&self) -> Sender<Box<dyn Report>>`
Returns the sender, called in Context

#### `get_receiver(&self) -> Arc<Mutex<Receiver<Box<dyn Report>>>>`
Provides access to the receiver that can be used to receive Report objects from the channel, called in unit test. 

## Context Struct 

The Context struct provides mechanisms for sending reports through channels and creating/managing new states. 

### Fields

#### `sender: Sender<Box<dyn Report>>`
Channel sender

### Methods 

#### `new(global_state: Arc<Mutex<GlobalState>>) -> Self`
Creates and returns a new Context instance. Initializes the `sender` by locking GlobalState and retrieving the sender.

#### `send_report<T: Report>(&self, report: T)`
Sends a report through the channel. It wraps the report in a Box<dyn Report> and sends it using the sender.

## Example Usage 

Main.rs has an example of how to use this. 

### General Implementation

Declare the global state 
`let global_state = Arc::new(Mutex::new(GlobalState::new()))`

Add report type to global state
```rust
{
    let mut global_state = global_state.lock().unwrap();
    global_state.register_report_type::<Incidence>("incidence_report.csv");
}
```

Start consumer thread
```rust
{
    let mut global_state = global_state.lock().unwrap();
    global_state.start_consumer_thread();
}
```

Handles vector 
`let mut handles = vec![]`

Send reports 
```rust
for i in 0..number_of_contexts {
        let global_state = Arc::clone(&global_state);
        let handle = thread::spawn(move || {
            let mut context = Context::new(global_state);
            for counter in 0..number_of_counters {
                let report_type = ReportType {
                    context_name: format!("Context {}", i),
                    counter,
                    timestamp: format!("2023-06-26 {}", counter),
                    new_cases: 150 + counter as u32,
                }
                context.send_report(report_type);
            }
        });
    handles.push(handle);
}
for handle in handles {
    handle.join().unwrap();
}

global_state.lock().unwrap().join_consumer_thread();
drop(global_state)
