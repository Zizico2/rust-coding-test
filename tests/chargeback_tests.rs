mod common;

use common::{account, run};
use rust_coding_test::domain::{Chargeback, ClientId, Deposit, Dispute, Resolve};
use rust_decimal::dec;
use std::collections::HashMap;

/// Spec: "the clients held funds and total funds should decrease by the amount
/// previously disputed. If a chargeback occurs the client's account should be
/// immediately frozen."
#[test]
fn chargeback_removes_funds_and_locks_account() {
    let engine = run(vec![
        Deposit::new(1.into(), 1.into(), dec!(100.0)).into(),
        Dispute::new(1.into(), 1.into()).into(),
        Chargeback::new(1.into(), 1.into()).into(),
    ]);

    let expected = HashMap::from([(ClientId::from(1), account(dec!(0.0), dec!(0.0), true))]);

    assert_eq!(engine.client_accounts().as_map(), &expected);
}

/// Spec: "if the tx isn't under dispute, you can ignore chargeback"
#[test]
fn chargeback_without_prior_dispute_is_ignored() {
    let engine = run(vec![
        Deposit::new(1.into(), 1.into(), dec!(100.0)).into(),
        Chargeback::new(1.into(), 1.into()).into(), // no dispute open
    ]);

    let expected = HashMap::from([(ClientId::from(1), account(dec!(100.0), dec!(0.0), false))]);

    assert_eq!(engine.client_accounts().as_map(), &expected);
}

/// Spec: "if the tx specified doesn't exist [...] you can ignore chargeback"
#[test]
fn chargeback_on_nonexistent_transaction_is_ignored() {
    let engine = run(vec![
        Deposit::new(1.into(), 1.into(), dec!(100.0)).into(),
        Chargeback::new(1.into(), 99.into()).into(),
    ]);

    let expected = HashMap::from([(ClientId::from(1), account(dec!(100.0), dec!(0.0), false))]);

    assert_eq!(engine.client_accounts().as_map(), &expected);
}

/// Spec: only the disputed amount is reversed - other deposits are unaffected.
#[test]
fn chargeback_preserves_remaining_balance_for_other_deposits() {
    let engine = run(vec![
        Deposit::new(1.into(), 1.into(), dec!(50.0)).into(),
        Deposit::new(1.into(), 2.into(), dec!(50.0)).into(),
        Dispute::new(1.into(), 1.into()).into(),
        Chargeback::new(1.into(), 1.into()).into(),
    ]);

    let expected = HashMap::from([(ClientId::from(1), account(dec!(50.0), dec!(0.0), true))]);

    assert_eq!(engine.client_accounts().as_map(), &expected);
}

/// Spec: "If the tx specified doesn't exist [...] you can ignore chargeback"
/// A chargeback referencing another client's tx should be ignored.
#[test]
fn chargeback_on_wrong_client_is_ignored() {
    let engine = run(vec![
        Deposit::new(1.into(), 1.into(), dec!(100.0)).into(),
        Dispute::new(1.into(), 1.into()).into(),
        Chargeback::new(2.into(), 1.into()).into(), // client 2 tries to chargeback client 1's tx
    ]);

    let expected = HashMap::from([
        (ClientId::from(1), account(dec!(0.0), dec!(100.0), false)),
        (ClientId::from(2), account(dec!(0.0), dec!(0.0), false)),
    ]);

    assert_eq!(engine.client_accounts().as_map(), &expected);
}

/// Spec: chargeback on a tx that was disputed then resolved (no re-dispute)
/// should be ignored because the tx is no longer under dispute.
#[test]
fn chargeback_after_resolve_without_redispute_is_ignored() {
    let engine = run(vec![
        Deposit::new(1.into(), 1.into(), dec!(100.0)).into(),
        Dispute::new(1.into(), 1.into()).into(),
        Resolve::new(1.into(), 1.into()).into(),
        Chargeback::new(1.into(), 1.into()).into(), // no active dispute → ignored
    ]);

    let expected = HashMap::from([(ClientId::from(1), account(dec!(100.0), dec!(0.0), false))]);

    assert_eq!(engine.client_accounts().as_map(), &expected);
}

/// Spec + Assumption 2: transaction cannot be re-disputed after chargeback.
#[test]
fn redispute_after_chargeback_is_ignored() {
    let engine = run(vec![
        Deposit::new(1.into(), 1.into(), dec!(100.0)).into(),
        Dispute::new(1.into(), 1.into()).into(),
        Chargeback::new(1.into(), 1.into()).into(),
        Dispute::new(1.into(), 1.into()).into(),
    ]);

    let expected = HashMap::from([(ClientId::from(1), account(dec!(0.0), dec!(0.0), true))]);

    assert_eq!(engine.client_accounts().as_map(), &expected);
}
