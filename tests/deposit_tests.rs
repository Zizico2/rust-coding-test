mod common;

use common::{account, run};
use rust_coding_test::domain::{ClientId, Deposit};
use rust_decimal::dec;
use std::collections::HashMap;

/// Spec: "A deposit is a credit to the client's asset account, meaning it should
/// increase the available and total funds of the client account"
#[test]
fn single_deposit_creates_account_with_correct_balance() {
    let engine = run(vec![Deposit::new(1.into(), 1.into(), dec!(50.0)).into()]);

    let expected = HashMap::from([(ClientId::from(1), account(dec!(50.0), dec!(0.0), false))]);

    assert_eq!(engine.client_accounts().as_map(), &expected);
}

/// Spec: deposits increase available funds - multiple deposits should accumulate.
#[test]
fn multiple_deposits_accumulate() {
    let engine = run(vec![
        Deposit::new(1.into(), 1.into(), dec!(10.0)).into(),
        Deposit::new(1.into(), 2.into(), dec!(10.0)).into(),
        Deposit::new(1.into(), 3.into(), dec!(10.0)).into(),
        Deposit::new(1.into(), 4.into(), dec!(10.0)).into(),
    ]);

    let expected = HashMap::from([(ClientId::from(1), account(dec!(40.0), dec!(0.0), false))]);

    assert_eq!(engine.client_accounts().as_map(), &expected);
}

/// Spec: "The client has a single asset account" and "There are multiple clients."
#[test]
fn deposits_for_multiple_clients_are_independent() {
    let engine = run(vec![
        Deposit::new(1.into(), 1.into(), dec!(100.0)).into(),
        Deposit::new(2.into(), 2.into(), dec!(200.0)).into(),
    ]);

    let expected = HashMap::from([
        (ClientId::from(1), account(dec!(100.0), dec!(0.0), false)),
        (ClientId::from(2), account(dec!(200.0), dec!(0.0), false)),
    ]);

    assert_eq!(engine.client_accounts().as_map(), &expected);
}

/// Spec: "a decimal value with a precision of up to four places past the decimal"
#[test]
fn deposit_with_decimal_precision() {
    let engine = run(vec![
        Deposit::new(1.into(), 1.into(), dec!(0.1234)).into(),
        Deposit::new(1.into(), 2.into(), dec!(0.8766)).into(),
    ]);

    let expected = HashMap::from([(ClientId::from(1), account(dec!(1.0000), dec!(0.0), false))]);

    assert_eq!(engine.client_accounts().as_map(), &expected);
}
