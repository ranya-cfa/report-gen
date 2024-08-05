# Option 2

## Overview 

This implementation has one channel per report type and one reader thread per report type. 

## Global State Struct

The GlobalState struct manages the state and coordination of report generation in the simulation. It provides functionality to register new report types, manage sender and receiver channels, and handle consumer threads. 

### Fields

#### `report_senders: HashMap<TypeId, Sender<Box<dyn Report + Send>>>`

A map that associates `TypeId` with a `Sender` for sending `Box<dyn Report + Send>` messages. This allows for the sending of reports of various types within the system.

- **Key**: `TypeId` - The unique identifier for the type of report.
- **Value**: `Sender<Box<dyn Report + Send>>` - A sender channel for dispatching report objects of the specified type.


#### `threads: Vec<JoinHandle<()>>`

A vector of thread handles. Each time a new thread is spawned, the handle is pushed to this vector. We iterate over this vector later to join all threads. 

### Methods 

#### `new() -> Self`

Creates and returns a new GlobalState instance with empty fields.

#### `setup_report<T: Report + 'static>(&mut self, filename: &str)`

Creates channel for specific report type. TheTypeId of the sender is inserted into the report_senders map. A new thread is spawned to process the incoming reports of this report type. A specific writer is created for the file associated with the report type, and the reports are serialized as they pass through the channel.  

`T` - The type of the report to be registered.
`filename` - The file path where the report of this type will be saved.

#### `get_report_sender<T: Report + 'static>(&self,) -> Option<&Sender<Box<dyn Report + Send>>>`

Returns the sender if the sender exists, None if not. Retrieves the sender based on the unique TypeId of the report type.  

#### `report_senders_is_empty(&self) -> bool`

Returns boolean value if the report_senders hashmap is empty. The map would be empty if no reports have been added yet or if the map has been cleared. This method is called in a unit test to ensure that the map is empty upon initialization. 

#### `join_threads(&mut self)`

The method waits for each thread to finish execution, and then discards the old list of handles.

#### `add_report<T: Report + 'static>(&mut self, filename: &str)`

When add_report is called, a report of a specific type is set up. `setup_report` is called for setting up report type `T`. 

`filename` - The file path where the report of this type will be saved.

## Context Struct 

The Context struct provides mechanisms for sending reports through channels and creating/managing new states. 

### Fields

#### `global_state: Arc<Mutex<GlobalState>>`

The global state shared across different contexts. Mutex is necessary to ensure thread safety. 

#### `report_senders: HashMap<TypeId, Sender<Box<dyn Report + Send>>>`

A map that associates `TypeId` with a `Sender` for sending `Box<dyn Report + Send>` messages. This allows for the sending of reports of various types within the system.

- **Key**: `TypeId` - The unique identifier for the type of report.
- **Value**: `Sender<Box<dyn Report + Send>>` - A sender channel for dispatching report objects of the specified type.

### Methods 

#### `new() -> Self`

Creates and returns a new Context instance with empty fields.

#### `release_report_item<T: Report + 'static>(&mut self, item: T)`

This function routes the report item to its appropriate channel. It determines the TypeId of the item, checks if the associated sender is known in the hashmap. If not, it will insert the associated sender into the hashmap. This is done to prevent unnecessarily repeatedly cloning the entire report_senders from GlobalState. Then, the item is sent through the sender. 

## Example Usage 

Main.rs has an example of how to use this. 

### General Implementation

Declare the global state 
`let global_state = Arc::new(Mutex::new(GlobalState::new()))`

Add report type to global state
`global_state.lock().unwrap().add_report::<ReportType>("report_type.csv")`

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
                context.release_report_item(report_type);
            }
        });
    handles.push(handle);
}
for handle in handles {
    handle.join().unwrap();
}

global_state.lock().unwrap().join_threads();
