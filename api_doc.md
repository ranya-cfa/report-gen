# Option 4

## Overview 

This implementation has one channel per report type and one reader thread per report type. 

This document describes the API that involves three main components: Reports, Global State, and Contexts. The system is designed to create, manage, and serialize various types of reports in a multithreaded environment.

## Reports
Reports represent different types of data collected during the simulation. Each report type must implement the `Report` trait in order to enable serialization. 

### Report Structs
Declare report types and relevant fields. `ReportType` indicates the type of report that is being declared and the `info` field is the information for that specific field. For example, for the `Incidence` report type, the info field might be renamed to `new_cases`, or for the `Death` report type, the info field might be renamed to `death` and contain either the number of deaths or death rate. This struct should essentially be modified to include the relevant fields that should be printed to the final CSV.  

```rust 
pub struct ReportType {
    pub context_name: String,
    pub counter: usize,
    pub timestamp: String,
    pub info: u32,
}
```

### Create report traits 
After declaring the report types, we must actually create the report traits by calling the `create_report_trait` macro and passing the name of the report type to it. The macro is used to automatically implement common functionality for each report type, such as `serialize`, which is used to convert the report data into CSV format. 
`create_report_trait!(ReportType)`

## Global State 
Global State manages the global settings, report types, and handles report serialization. 

### Declare global state and add reports 
Declare the global state with `new()`. This state will be shared across different threads, so we wrap it in Arc<Mutex<>> to ensure safe concurrent access and modification.
`let global_state = Arc::new(Mutex::new(GlobalState::new()))`
Next, add the desired report types to the global state. This is required to setup the report type. Each report type is associated with a specific CSV file for output. Specify the output file in the `add_report` parameter. 
`global_state.lock().unwrap().add_report::<ReportType>("report_type.csv")`

### Join consumer threads at end of program
To ensure all consumer threads complete their tasks before the program ends, call the `join_threads` method on the global state. This method waits for all consumer threads to finish processing their reports.
`global_state.lock().unwrap().join_threads()`

## Context
Context represents a specific execution environment within the simulation, responsible for generating and sending reports. It interacts with `GlobalState` to send reports. Each Context instance holds a clone of `Arc<Mutex<GlobalState>>`. This allows each context to share the same GlobalState while ensuring that access is thread-safe.

### Create context 
To create a new context, use the `new()` method and pass the global state to it. This initializes a context that can send reports through the shared global state.
`let mut context = Context::new(global_state)`

### Release report item
To send a report to the GlobalState, create an instance of the report type with the required fields and call the release_report_item method on the context.
```rust
let report_type = ReportType {
    context_name: ,
    counter,
    timestamp: ,
    info: ,
    }
context.release_report_item(report_type);
```