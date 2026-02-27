mod common;

use common::{account, run};
use rust_coding_test::domain::{Chargeback, ClientId, Deposit, Dispute, Resolve, Withdrawal};
use rust_decimal::dec;
use std::collections::HashMap;

/// Spec: "The client has a single asset account" - client isolation.
#[test]
fn chargeback_on_one_client_does_not_affect_another() {
    let engine = run(vec![
        Deposit::new(1.into(), 1.into(), dec!(100.0)).into(),
        Deposit::new(2.into(), 2.into(), dec!(200.0)).into(),
        Dispute::new(1.into(), 1.into()).into(),
        Chargeback::new(1.into(), 1.into()).into(),
    ]);

    let expected = HashMap::from([
        (ClientId::from(1), account(dec!(0.0), dec!(0.0), true)),
        (ClientId::from(2), account(dec!(200.0), dec!(0.0), false)),
    ]);

    assert_eq!(engine.client_accounts().as_map(), &expected);
}

/// Spec: transactions to different clients can be interleaved; order is chronological.
#[test]
fn interleaved_transactions_for_multiple_clients() {
    let engine = run(vec![
        Deposit::new(1.into(), 1.into(), dec!(100.0)).into(),
        Deposit::new(2.into(), 2.into(), dec!(200.0)).into(),
        Withdrawal::new(1.into(), 3.into(), dec!(30.0)).into(),
        Withdrawal::new(2.into(), 4.into(), dec!(50.0)).into(),
        Deposit::new(1.into(), 5.into(), dec!(10.0)).into(),
    ]);

    let expected = HashMap::from([
        (ClientId::from(1), account(dec!(80.0), dec!(0.0), false)),
        (ClientId::from(2), account(dec!(150.0), dec!(0.0), false)),
    ]);

    assert_eq!(engine.client_accounts().as_map(), &expected);
}

/// Spec: dispute then resolve should leave balance unchanged (round-trip).
#[test]
fn full_dispute_resolve_cycle_leaves_balance_intact() {
    let engine = run(vec![
        Deposit::new(1.into(), 1.into(), dec!(100.0)).into(),
        Deposit::new(1.into(), 2.into(), dec!(50.0)).into(),
        Dispute::new(1.into(), 1.into()).into(),
        Resolve::new(1.into(), 1.into()).into(),
    ]);

    let expected = HashMap::from([(ClientId::from(1), account(dec!(150.0), dec!(0.0), false))]);

    assert_eq!(engine.client_accounts().as_map(), &expected);
}

/// Spec: dispute then chargeback removes funds and locks account.
#[test]
fn full_dispute_chargeback_cycle() {
    let engine = run(vec![
        Deposit::new(1.into(), 1.into(), dec!(100.0)).into(),
        Dispute::new(1.into(), 1.into()).into(),
        Chargeback::new(1.into(), 1.into()).into(),
    ]);

    let expected = HashMap::from([(ClientId::from(1), account(dec!(0.0), dec!(0.0), true))]);

    assert_eq!(engine.client_accounts().as_map(), &expected);
}

/// Assumption 2 + Spec: a resolved tx can be re-disputed and then charged back.
#[test]
fn re_dispute_after_resolve_then_chargeback() {
    let engine = run(vec![
        Deposit::new(1.into(), 1.into(), dec!(100.0)).into(),
        Dispute::new(1.into(), 1.into()).into(),
        Resolve::new(1.into(), 1.into()).into(),
        Dispute::new(1.into(), 1.into()).into(), // re-dispute
        Chargeback::new(1.into(), 1.into()).into(), // chargeback the re-dispute
    ]);

    let expected = HashMap::from([(ClientId::from(1), account(dec!(0.0), dec!(0.0), true))]);

    assert_eq!(engine.client_accounts().as_map(), &expected);
}
