# Define report types

## Trait Definition 
Add methods that the reports will use. `serialize` is an example of one.
```rust
pub trait Report: Send + 'static {
    fn serialize(&self, writer: &mut Writer<File>);
}
```
## Report Structs
Declare report types and relevant fields

```rust 
pub struct ReportType {
    pub context_name: String,
    pub counter: usize,
    pub timestamp: String,
    pub info: u32,
}
```

## Macro for Trait Implementation 
The macro simplifies the implementation of the `Report` trait for different report types.

```rust
macro_rules! create_report_trait {
    ($name:ident) => {
        impl Report for $name {
            fn serialize(&self, writer: &mut Writer<File>) {
                writer.serialize(self).unwrap();
            }
        }
    };
}
```

## Create report traits 
Create traits for all report types 
`create_report_trait!(ReportType)`

# Declare global state and add reports 
```rust
use ixa::GlobalState;

let global_state = Arc::new(Mutex::new(GlobalState::new()))
global_state.lock().unwrap().add_report::<ReportType>("report_type.csv")
```

# Create context 
```rust
use ixa::Context;

let mut context = Context::new(global_state);
```

# Release report items 
```rust
let report_type = ReportType {
    context_name: ,
    counter,
    timestamp: ,
    info: ,
    }
context.release_report_item(report_type);
```

# Cleanup
Ensure all threads are joined and resources are cleaned up