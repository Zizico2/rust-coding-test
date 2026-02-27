use std::fs::File;

use clap::Parser;

use rust_coding_test::engine::PaymentsEngine;
use rust_coding_test::output;
use rust_coding_test::parsing;

fn main() {
    let args = Arguments::parse();
    if let Some(log_level) = args.log_level {
        tracing_subscriber::fmt().with_max_level(log_level).init();
    }

    let file_path = args.input_file;

    let file = File::open(file_path).unwrap();

    let mut rdr = csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        .from_reader(file);

    let transaction_iter = parsing::deserialize_csv(&mut rdr);

    let mut engine = PaymentsEngine::new();
    engine.process_transactions(transaction_iter);

    let client_accounts = engine.client_accounts();
    output::print_accounts(client_accounts, std::io::stdout());
}

#[derive(Parser)]
struct Arguments {
    input_file: String,
    log_level: Option<tracing::Level>,
}
