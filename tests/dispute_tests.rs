mod common;

use common::{account, run};
use rust_coding_test::domain::{Chargeback, ClientId, Deposit, Dispute, Resolve, Withdrawal};
use rust_decimal::dec;
use std::collections::HashMap;

/// Spec: "the clients available funds should decrease by the amount disputed,
/// their held funds should increase by the amount disputed"
#[test]
fn dispute_moves_funds_to_held() {
    let engine = run(vec![
        Deposit::new(1.into(), 1.into(), dec!(100.0)).into(),
        Dispute::new(1.into(), 1.into()).into(),
    ]);

    let expected = HashMap::from([(ClientId::from(1), account(dec!(0.0), dec!(100.0), false))]);

    assert_eq!(engine.client_accounts().as_map(), &expected);
}

/// Spec: "If the tx specified by the dispute doesn't exist you can ignore it"
#[test]
fn dispute_on_nonexistent_transaction_is_ignored() {
    let engine = run(vec![
        Deposit::new(1.into(), 1.into(), dec!(100.0)).into(),
        Dispute::new(1.into(), 99.into()).into(), // tx 99 doesn't exist
    ]);

    let expected = HashMap::from([(ClientId::from(1), account(dec!(100.0), dec!(0.0), false))]);

    assert_eq!(engine.client_accounts().as_map(), &expected);
}

/// Assumption 4: client 2's account is created even though the dispute fails.
/// The dispute itself is ignored because tx 1 belongs to client 1.
#[test]
fn dispute_on_wrong_client_is_ignored() {
    let engine = run(vec![
        Deposit::new(1.into(), 1.into(), dec!(100.0)).into(),
        Dispute::new(2.into(), 1.into()).into(),
    ]);

    let expected = HashMap::from([
        (ClientId::from(1), account(dec!(100.0), dec!(0.0), false)),
        (ClientId::from(2), account(dec!(0.0), dec!(0.0), false)),
    ]);

    assert_eq!(engine.client_accounts().as_map(), &expected);
}

/// Spec (implied): a tx already under dispute cannot be disputed again.
#[test]
fn duplicate_dispute_on_same_transaction_is_ignored() {
    // The second dispute on the same tx must be rejected; held stays at 100, not 200.
    let engine = run(vec![
        Deposit::new(1.into(), 1.into(), dec!(100.0)).into(),
        Dispute::new(1.into(), 1.into()).into(),
        Dispute::new(1.into(), 1.into()).into(),
    ]);

    let expected = HashMap::from([(ClientId::from(1), account(dec!(0.0), dec!(100.0), false))]);

    assert_eq!(engine.client_accounts().as_map(), &expected);
}

/// Spec: dispute references a specific tx - only that amount is held.
#[test]
fn dispute_partial_deposit_leaves_remaining_available() {
    let engine = run(vec![
        Deposit::new(1.into(), 1.into(), dec!(30.0)).into(),
        Deposit::new(1.into(), 2.into(), dec!(70.0)).into(),
        Dispute::new(1.into(), 1.into()).into(), // only dispute the first deposit
    ]);

    let expected = HashMap::from([(ClientId::from(1), account(dec!(70.0), dec!(30.0), false))]);

    assert_eq!(engine.client_accounts().as_map(), &expected);
}

/// Assumption 1: only deposits can be disputed. Disputing a withdrawal tx is a no-op.
#[test]
fn dispute_on_withdrawal_tx_id_is_ignored() {
    let engine = run(vec![
        Deposit::new(1.into(), 1.into(), dec!(100.0)).into(),
        Withdrawal::new(1.into(), 2.into(), dec!(40.0)).into(),
        Dispute::new(1.into(), 2.into()).into(), // tx 2 is a withdrawal
    ]);

    let expected = HashMap::from([(ClientId::from(1), account(dec!(60.0), dec!(0.0), false))]);

    assert_eq!(engine.client_accounts().as_map(), &expected);
}

/// Spec: multiple different deposits can be under dispute at the same time.
#[test]
fn multiple_disputes_on_different_txs() {
    let engine = run(vec![
        Deposit::new(1.into(), 1.into(), dec!(30.0)).into(),
        Deposit::new(1.into(), 2.into(), dec!(70.0)).into(),
        Deposit::new(1.into(), 3.into(), dec!(50.0)).into(),
        Dispute::new(1.into(), 1.into()).into(), // dispute 30
        Dispute::new(1.into(), 2.into()).into(), // dispute 70
    ]);

    // available = 150 - 30 - 70 = 50, held = 30 + 70 = 100
    let expected = HashMap::from([(ClientId::from(1), account(dec!(50.0), dec!(100.0), false))]);

    assert_eq!(engine.client_accounts().as_map(), &expected);
}

/// Complex scenario: 4 deposits, 4 disputes, mixed outcomes:
///   tx 1 (10)  → disputed → resolved      (funds return to available)
///   tx 2 (20)  → disputed → chargebacked   (funds removed, account locked)
///   tx 3 (30)  → disputed → left in held   (still held)
///   tx 4 (40)  → not disputed              (stays available)
#[test]
fn interleaved_disputes_with_mixed_outcomes() {
    let engine = run(vec![
        Deposit::new(1.into(), 1.into(), dec!(10.0)).into(),
        Deposit::new(1.into(), 2.into(), dec!(20.0)).into(),
        Deposit::new(1.into(), 3.into(), dec!(30.0)).into(),
        Deposit::new(1.into(), 4.into(), dec!(40.0)).into(),
        // total deposited = 100, available = 100
        Dispute::new(1.into(), 1.into()).into(), // hold 10
        Dispute::new(1.into(), 2.into()).into(), // hold 20
        Dispute::new(1.into(), 3.into()).into(), // hold 30
        // available = 40, held = 60
        Resolve::new(1.into(), 1.into()).into(), // release 10 back to available
        // available = 50, held = 50
        Chargeback::new(1.into(), 2.into()).into(), // remove 20 from held+total, locks account
                                                    // available = 50, held = 30, total = 80, locked = true
    ]);

    let expected = HashMap::from([(ClientId::from(1), account(dec!(50.0), dec!(30.0), true))]);

    assert_eq!(engine.client_accounts().as_map(), &expected);
}

/// Spec: "total funds should remain the same" during dispute - total = available + held.
#[test]
fn balance_total_equals_available_plus_held() {
    // 100 + 50 deposited; tx 1 (100) disputed → available = 50, held = 100, total = 150.
    let engine = run(vec![
        Deposit::new(1.into(), 1.into(), dec!(100.0)).into(),
        Deposit::new(1.into(), 2.into(), dec!(50.0)).into(),
        Dispute::new(1.into(), 1.into()).into(),
    ]);

    let expected = HashMap::from([(ClientId::from(1), account(dec!(50.0), dec!(100.0), false))]);

    assert_eq!(engine.client_accounts().as_map(), &expected);
}

/// Assumption 5: disputing a deposit that was partially withdrawn can result
/// in a negative available balance, representing a debt to the partner.
#[test]
fn dispute_after_partial_withdrawal_allows_negative_available() {
    // Deposit 100, withdraw 60, dispute the original deposit of 100.
    // available = 100 - 60 - 100 = -60, held = 100, total = 40.
    let engine = run(vec![
        Deposit::new(1.into(), 1.into(), dec!(100.0)).into(),
        Withdrawal::new(1.into(), 2.into(), dec!(60.0)).into(),
        Dispute::new(1.into(), 1.into()).into(),
    ]);

    let expected = HashMap::from([(ClientId::from(1), account(dec!(-60.0), dec!(100.0), false))]);

    assert_eq!(engine.client_accounts().as_map(), &expected);
}
/// Assumption 5: disputing and charging back a deposit that was partially withdrawn can result
/// in a negative available and total balance, representing a debt to the partner.
#[test]
fn dispute_and_chargeback_after_partial_withdrawal_allows_negative_available() {
    // Deposit 100, withdraw 60, dispute the original deposit of 100.
    // available = 100 - 60 - 100 = -60, held = 100, total = 40.
    let engine = run(vec![
        Deposit::new(1.into(), 1.into(), dec!(100.0)).into(),
        Withdrawal::new(1.into(), 2.into(), dec!(60.0)).into(),
        Dispute::new(1.into(), 1.into()).into(),
        Chargeback::new(1.into(), 1.into()).into(),
    ]);

    let expected = HashMap::from([(ClientId::from(1), account(dec!(-60.0), dec!(0.0), true))]);

    assert_eq!(engine.client_accounts().as_map(), &expected);
}
