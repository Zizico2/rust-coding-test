mod common;

use common::{account, run};
use rust_coding_test::domain::{Chargeback, ClientId, Deposit, Dispute, Resolve, Withdrawal};
use rust_decimal::dec;
use std::collections::HashMap;

/// Assumption 3: a locked (frozen) account rejects deposits and withdrawals.
/// Spec only says "account should be immediately frozen" - exact scope is assumed.
#[test]
fn locked_account_ignores_further_deposits() {
    let engine = run(vec![
        Deposit::new(1.into(), 1.into(), dec!(100.0)).into(),
        Dispute::new(1.into(), 1.into()).into(),
        Chargeback::new(1.into(), 1.into()).into(),
        Deposit::new(1.into(), 2.into(), dec!(500.0)).into(), // must be ignored
    ]);

    let expected = HashMap::from([(ClientId::from(1), account(dec!(0.0), dec!(0.0), true))]);

    assert_eq!(engine.client_accounts().as_map(), &expected);
}

/// Assumption 3: locked account rejects withdrawals.
#[test]
fn locked_account_ignores_withdrawals() {
    let engine = run(vec![
        Deposit::new(1.into(), 1.into(), dec!(100.0)).into(),
        Deposit::new(1.into(), 2.into(), dec!(50.0)).into(),
        Dispute::new(1.into(), 1.into()).into(),
        Chargeback::new(1.into(), 1.into()).into(),
        Withdrawal::new(1.into(), 3.into(), dec!(50.0)).into(), // must be ignored
    ]);

    let expected = HashMap::from([(ClientId::from(1), account(dec!(50.0), dec!(0.0), true))]);

    assert_eq!(engine.client_accounts().as_map(), &expected);
}

/// Assumption 3: locked account still allows disputes.
#[test]
fn locked_account_allows_disputes() {
    let engine = run(vec![
        Deposit::new(1.into(), 1.into(), dec!(100.0)).into(),
        Deposit::new(1.into(), 2.into(), dec!(50.0)).into(),
        Dispute::new(1.into(), 1.into()).into(),
        Chargeback::new(1.into(), 1.into()).into(),
        Dispute::new(1.into(), 2.into()).into(), // allowed on locked account
    ]);

    let expected = HashMap::from([(ClientId::from(1), account(dec!(0.0), dec!(50.0), true))]);

    assert_eq!(engine.client_accounts().as_map(), &expected);
}

/// Assumption 3: locked account still allows resolves.
#[test]
fn locked_account_allows_resolves() {
    let engine = run(vec![
        Deposit::new(1.into(), 1.into(), dec!(100.0)).into(),
        Deposit::new(1.into(), 2.into(), dec!(50.0)).into(),
        Dispute::new(1.into(), 1.into()).into(),
        Dispute::new(1.into(), 2.into()).into(),
        Chargeback::new(1.into(), 1.into()).into(), // locks account
        Resolve::new(1.into(), 2.into()).into(), // allowed - locked only blocks deposits/withdrawals
    ]);

    let expected = HashMap::from([(ClientId::from(1), account(dec!(50.0), dec!(0.0), true))]);

    assert_eq!(engine.client_accounts().as_map(), &expected);
}

/// Assumption 3: locked account still allows chargebacks.
#[test]
fn locked_account_allows_chargeback() {
    let engine = run(vec![
        Deposit::new(1.into(), 1.into(), dec!(100.0)).into(),
        Deposit::new(1.into(), 2.into(), dec!(50.0)).into(),
        Dispute::new(1.into(), 1.into()).into(),
        Dispute::new(1.into(), 2.into()).into(),
        Chargeback::new(1.into(), 1.into()).into(), // locks account
        Chargeback::new(1.into(), 2.into()).into(), // allowed - locked only blocks deposits/withdrawals
    ]);

    let expected = HashMap::from([(ClientId::from(1), account(dec!(0.0), dec!(0.0), true))]);

    assert_eq!(engine.client_accounts().as_map(), &expected);
}
