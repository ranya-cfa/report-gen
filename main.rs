fn main() {
    let context = Context::new();

    context.add_report::<Incidence>("incidence_report.csv");
    context.add_report::<Death>("death_report.csv");

    context.release_report_item(Incidence {
        timestamp: "2023-06-26 0".to_string(),
        new_cases: 150,
    });
    context.release_report_item(Death {
        timestamp: "2023-06-26 0".to_string(),
        deaths: 5,
    });

    context.execute();
}