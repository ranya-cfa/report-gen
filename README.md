Framework Skeleton

A global state (global_state.rs) manages report generation across multiple simulation 
contexts. It maintains a map of report senders, allowing different report types to be 
registered and accessed globally. The hashmap maps TypeId to a transaction sender. This
allows each report type to be associated with a sender. The global state creates 
report traits, producer threads, and consumer threads. When a new report type is added 
with add_report, a new producer thread is created.   

Context (context.rs) releases report items into the corresponding channels. 

In main.rs, different report types are added to the global state and report items 
are released for processing. 