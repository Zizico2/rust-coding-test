mod common;

use common::{account, run};
use rust_coding_test::domain::{ClientId, Deposit, Dispute, Resolve};
use rust_decimal::dec;
use std::collections::HashMap;

/// Spec: "the clients held funds should decrease by the amount no longer disputed,
/// their available funds should increase by the amount no longer disputed"
#[test]
fn resolve_releases_held_funds_back_to_available() {
    let engine = run(vec![
        Deposit::new(1.into(), 1.into(), dec!(100.0)).into(),
        Dispute::new(1.into(), 1.into()).into(),
        Resolve::new(1.into(), 1.into()).into(),
    ]);

    let expected = HashMap::from([(ClientId::from(1), account(dec!(100.0), dec!(0.0), false))]);

    assert_eq!(engine.client_accounts().as_map(), &expected);
}

/// Spec: "if the tx isn't under dispute, you can ignore the resolve"
#[test]
fn resolve_without_prior_dispute_is_ignored() {
    let engine = run(vec![
        Deposit::new(1.into(), 1.into(), dec!(100.0)).into(),
        Resolve::new(1.into(), 1.into()).into(), // no dispute open
    ]);

    let expected = HashMap::from([(ClientId::from(1), account(dec!(100.0), dec!(0.0), false))]);

    assert_eq!(engine.client_accounts().as_map(), &expected);
}

/// Spec: "If the tx specified doesn't exist [...] you can ignore the resolve"
#[test]
fn resolve_on_nonexistent_transaction_is_ignored() {
    let engine = run(vec![
        Deposit::new(1.into(), 1.into(), dec!(100.0)).into(),
        Resolve::new(1.into(), 99.into()).into(), // tx 99 doesn't exist
    ]);

    let expected = HashMap::from([(ClientId::from(1), account(dec!(100.0), dec!(0.0), false))]);

    assert_eq!(engine.client_accounts().as_map(), &expected);
}

/// Assumption 2: a resolved tx can be re-disputed.
/// The spec does not explicitly allow or forbid this.
#[test]
fn after_resolve_dispute_can_be_reopened() {
    let engine = run(vec![
        Deposit::new(1.into(), 1.into(), dec!(100.0)).into(),
        Dispute::new(1.into(), 1.into()).into(),
        Resolve::new(1.into(), 1.into()).into(),
        Dispute::new(1.into(), 1.into()).into(), // re-dispute
    ]);

    let expected = HashMap::from([(ClientId::from(1), account(dec!(0.0), dec!(100.0), false))]);

    assert_eq!(engine.client_accounts().as_map(), &expected);
}

/// Spec: "If the tx specified doesn't exist [...] you can ignore the resolve"
/// A resolve referencing another client's tx should be ignored.
#[test]
fn resolve_on_wrong_client_is_ignored() {
    let engine = run(vec![
        Deposit::new(1.into(), 1.into(), dec!(100.0)).into(),
        Dispute::new(1.into(), 1.into()).into(),
        Resolve::new(2.into(), 1.into()).into(), // client 2 tries to resolve client 1's dispute
    ]);

    let expected = HashMap::from([
        (ClientId::from(1), account(dec!(0.0), dec!(100.0), false)),
        (ClientId::from(2), account(dec!(0.0), dec!(0.0), false)),
    ]);

    assert_eq!(engine.client_accounts().as_map(), &expected);
}
