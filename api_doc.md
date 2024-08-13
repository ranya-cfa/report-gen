Note: For each context, this implementation outputs a file for each report type.. 

## Overview 

When a model is executed, you want to report information about the state of each simulation to one or more summary files (a "report"). For example, if you have a respiratory disease model that infects people and some die, you might have two reports: an incidence report that records each time a person is infected, and a death report that records when they die. 

This utility is designed to create, manage, and serialize reports in a multithreaded environment, where each simulation runs in a separate thread but outputs to a single set of reports.
Each report type is defined by a struct and a `Context` instance that manages file writing and shared states for each simulation.

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

## Context
`Context` represents the execution environment and shared state for a single simulation. Each context will write its own set of reports. 

### Registering Contexts
Declare the name of the simulation under the variable `context_name`. Create an instance of `Context`, passing `context_name` to it.

``` rust
let context_name = "context_one";
let mut context = Context::new(context_name.clone());
```
### `add_report::<ReportType>(short_name: &str)`

Call `add_report` with each report type, passing the name of the report type. The report type name is used for file naming to distinguish what data each output file points to.  

```rust
context.add_report::<Incidence>("Incidence");
context.add_report::<Death>("Death");
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

## Usage in multi-threaded scenario
In a multithreaded scenario, you will spawn a thread and create a new context for each simulation. Again, 
each context will output its own set of report files. At the end, we must merge these report files to get one file per report type. 

### Registering multiple Contexts
Since the user will determine the different simulations they want to run, they must store the names of the simulations so that those names can be integrated into the resulting report filename. We will iterate over this list to run all necessary simulations. 

``` rust
let context_names = vec!["context_0", "context_1", "context_2", "context_3"];
for context_name in context_names {
    let context_name = context_name.to_string();
    let handle = thread::spawn(move || {
        let mut context = Context::new(context_name.clone());
    ...
    });
}
```
`add_report` and `send_report` function the same in a multi-threaded scenario as they do in a single-threaded scenario. 
Refer to previous documentation.

### Merge multiple report files into one per report type
Call an external script to merge all CSVs for a report type into one file. So at the end, there will be one file per report type. 