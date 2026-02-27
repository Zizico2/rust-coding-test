mod common;

use common::run;

use rust_coding_test::{
    domain::{Deposit, Withdrawal},
    output, parsing,
};
use rust_decimal::dec;

const OUTPUT: &str = include_str!("io_tests/test_output.csv");
const INPUT: &[u8] = include_bytes!("io_tests/test_input.csv");

// test output
#[test]
fn test_output() -> anyhow::Result<()> {
    let transactions = vec![
        Deposit::new(1.into(), 1.into(), dec!(1.0)).into(),
        Deposit::new(1.into(), 3.into(), dec!(2.0)).into(),
        Withdrawal::new(1.into(), 4.into(), dec!(1.5)).into(),
    ];

    let engine = run(transactions);
    let client_accounts = engine.client_accounts();

    let mut output = Vec::new();

    output::print_accounts(client_accounts, &mut output)?;

    let output = String::from_utf8(output)?;

    assert_eq!(output, OUTPUT);

    Ok(())
}

// test input
#[test]
fn test_input() {
    let mut rdr = csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        .from_reader(INPUT);

    let transactions = parsing::deserialize_csv(&mut rdr).collect::<Vec<_>>();

    let expected = vec![
        Deposit::new(1.into(), 1.into(), dec!(1.0)).into(),
        Deposit::new(1.into(), 3.into(), dec!(2.0)).into(),
        Withdrawal::new(1.into(), 4.into(), dec!(1.5)).into(),
    ];

    assert_eq!(transactions, expected);
}
