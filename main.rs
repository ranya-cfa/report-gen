fn main() {
    let global_state = GLOBAL_STATE.lock().unwrap();

    global_state.add_report::<Incidence>("incidence_report.csv");
    global_state.add_report::<Death>("death_report.csv");

    global_state.start_producer_threads();
}