# Option 4

## Overview 

This implementation has one channel per report type and one reader thread per report type. 

This document describes the API that involves three main components: Reports, Global State, and Contexts. The system is designed to create, manage, and serialize various types of reports in a multithreaded environment.

## Reports
Reports represent different types of data collected during the simulation. Each report type must implement the `Report` trait in order to enable serialization. 

### Report Structs
Declare report types and relevant fields

```rust 
pub struct ReportType {
    pub context_name: String,
    pub counter: usize,
    pub timestamp: String,
    pub info: u32,
}
```

### Create report traits 
Create traits for all report types 
`create_report_trait!(ReportType)`

## Global State 
Global State manages the global settings, report types, and handles report serialization. 

### Declare global state and add reports 
```rust
let global_state = Arc::new(Mutex::new(GlobalState::new()))
global_state.lock().unwrap().add_report::<ReportType>("report_type.csv")
```
### Join consumer threads at end of program
`global_state.lock().unwrap().join_threads()`

## Context
Context represents a specific execution environment within the simulation, responsible for generating and sending reports. It interacts with `GlobalState` to send reports. Each Context instance holds a clone of `Arc<Mutex<GlobalState>>`. This allows each context to share the same GlobalState while ensuring that access is thread-safe.

### Create context 
`let mut context = Context::new(global_state)`

### Release report item
Send report to GlobalState
```rust
let report_type = ReportType {
    context_name: ,
    counter,
    timestamp: ,
    info: ,
    }
context.release_report_item(report_type);
```