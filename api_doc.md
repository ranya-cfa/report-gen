# Option 2
Note: this implementation has one channel, with all report types written onto it and one reader thread that demuxes them.

## Overview 

When a model is executed, you want to report information about the state of each simulation to one or more summary files (a "report"). For example, if you have a respiratory disease model that infects people and some die, you might have two reports: an incidence report that records each time a person is infected, and a death report that records when they die. 

This utility is designed to create, manage, and serialize reports in a multithreaded environment, where each simulation runs in a separate thread but outputs to a single set of reports.
Each report type is defined by a struct, a `GlobalState` instance is created to manage all the thread communication and file writing, and as events occur which need to be recorded, you will call a method on the `Context` to relay them back to `GlobalState`.

## Reports
Reports are files that record different types of outputs generated during the simulation, such as the time of each infection. You must define a struct for each report where the fields correspond to the columns you want in the resulting CSV file. For example, an `Incidence` report might have a `timestamp` column to record the time of infection, as well as an `info` column for additional metadata. 

For the following example:

```rust 
pub struct Incidence {
    pub timestamp: String,
    pub info: u32,
}
```
the resulting `incidence.csv` will look something like this:

```
timestamp,info
2023-08-05,55
2023-08-06,48
```

### `create_report_trait!`

Because outputs are serialized to files, your report type must implement some common functionality
such as `serialize`, which is used to convert the report data into CSV format. You can do this by implementing the `Report` trait, but the easiest way to do this is to use `create_report_trait!`. For the above example:

```rust 
create_report_trait!(Incidence)
```

## GlobalState 
`GlobalState` is responsible for thread management between the simulations and the writer threads. When running a multi-threaded experiment, you need to create a single `GlobalState` instance that is shared across all your simulations. 

### `register_report_type::<ReportType>(filename: &str)`

Create an instance of `GlobalState`, and call `register_report_type` with each report type along with a filename.

Place the code that locks the `Mutex` inside a code block to ensure that the lock is released as soon as the block is exited. This is essential for controlling the scope of the lock. If the `global_state` lock is held while performing multiple operations, any other threads that need to access `global_state` would be forced to wait. For instance, if another thread wanted to register a new report type, it would be blocked until the lock is released. 

See an example of `register_report_type` below:

```rust
let global_state = Arc::new(Mutex::new(GlobalState::new()));
    {
        let mut state = global_state.lock().unwrap();
        state.register_report_type::<Incidence>("incidence_report.csv");
        state.register_report_type::<Death>("death_report.csv");
    }
```

See `Context` documentation for how to add a `GlobalState` to each simulation.

### `start_consumer_thread()`

Start the consumer thread that will read the reports. This function must be called before any reports are sent and is necessary for the report to be written to the correct file. Again, a new block is used to lock `global_state`. 

``` rust
{
        let mut state = global_state.lock().unwrap();
        state.start_consumer_thread();
    }
```

### `join_consumer_thread()`
When all simulations have finished executing, call `join_consumer_thread()` to close all channels and finish writing reports. You need to call this function or the program will execute without guaranteeing that all reports have been properly written to the file. 

``` rust
global_state.lock().unwrap().join_consumer_thread();
```

## Context
`Context` represents the execution environment and shared state for a single simulation. In a multithreaded scenario, you will spawn a thread and create a new context for each simulation. Each context will need a copy of a thread safe `GlobalState` pointer in order to write to a shared set of reports.

### Registering GlobalState
Since `GlobalState` will be shared across different threads, you must wrap it in `Arc<Mutex<>>` to ensure safe concurrent access and modification. Whenever you create a new simulation, you will need to clone the pointer and pass it to the `Context` for that simulation:

```rust
let global_state = Arc::new(Mutex::new(GlobalState::new()));
for i in 0..n {
    let global_state = global_state.clone();
    let handle = thread::spawn(move || {
        let mut context = Context::new(global_state);
        ...
    })  
}
```

### `send_report(item: T)`

Anytime something happens (ie. a person gets infected) in the model that you want to record, you can 
use `send_report` to write a new row to a report. Note that you will need to call the method
with the appropriate report type:

```rust
let new_row = Incidence {
    timestamp: "2023-06-26 0".to_string(),
    info: 100,
}
context.send_report(new_row);
```