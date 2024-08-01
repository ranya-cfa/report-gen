One channel, with all report types written onto it, one reader thread that demuxes them.

Global state creates a sender and receiver, which are necessary to establish the channel. It also has typeid_to_writer, which is a map of the report type to the associated writer. Additionally, it creates one consumer thread. 

In start_consumer_thread, reports received through the channel are processed. It creates a single thread that processes all incoming reports regardless of their type. The receiver and typeid_to_writer are cloned for use in a new thread. The new thread processes incoming reports. It gets the type id of the received report, and if a writer exists for this type id, it serializes the report. The handle of the spawned thread is stored in consumer_thread. 

In send_report in context.rs, it accepts a report of type T that implements the Report trait. It then sends the report through the sender to the global state's channel.
