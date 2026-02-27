mod common;

use common::{account, run};
use rust_coding_test::domain::{ClientId, Deposit, Dispute, Withdrawal};
use rust_decimal::dec;
use std::collections::HashMap;

/// Spec: "A withdraw is a debit to the client's asset account, meaning it should
/// decrease the available and total funds of the client account"
#[test]
fn withdrawal_reduces_available_balance() {
    let engine = run(vec![
        Deposit::new(1.into(), 1.into(), dec!(100.0)).into(),
        Withdrawal::new(1.into(), 2.into(), dec!(40.0)).into(),
    ]);

    let expected = HashMap::from([(ClientId::from(1), account(dec!(60.0), dec!(0.0), false))]);

    assert_eq!(engine.client_accounts().as_map(), &expected);
}

/// Spec: withdrawals decrease available funds - edge case: exact balance.
#[test]
fn withdrawal_of_exact_balance_leaves_zero() {
    let engine = run(vec![
        Deposit::new(1.into(), 1.into(), dec!(50.0)).into(),
        Withdrawal::new(1.into(), 2.into(), dec!(50.0)).into(),
    ]);

    let expected = HashMap::from([(ClientId::from(1), account(dec!(0.0), dec!(0.0), false))]);

    assert_eq!(engine.client_accounts().as_map(), &expected);
}

/// Spec: "If a client does not have sufficient available funds the withdrawal
/// should fail and the total amount of funds should not change"
#[test]
fn withdrawal_exceeding_balance_is_ignored() {
    let engine = run(vec![
        Deposit::new(1.into(), 1.into(), dec!(30.0)).into(),
        Withdrawal::new(1.into(), 2.into(), dec!(100.0)).into(),
    ]);

    let expected = HashMap::from([(ClientId::from(1), account(dec!(30.0), dec!(0.0), false))]);

    assert_eq!(engine.client_accounts().as_map(), &expected);
}

/// Assumption 4: account is created with 0 balance even if first tx is not a deposit.
/// The withdrawal itself fails (insufficient funds), but the account exists.
#[test]
fn withdrawal_without_prior_deposit_is_ignored() {
    let engine = run(vec![Withdrawal::new(1.into(), 1.into(), dec!(10.0)).into()]);

    let expected = HashMap::from([(ClientId::from(1), account(dec!(0.0), dec!(0.0), false))]);

    assert_eq!(engine.client_accounts().as_map(), &expected);
}

/// Spec: "If a client does not have sufficient available funds the withdrawal
/// should fail." Held funds reduce available even though total is high enough.
#[test]
fn withdrawal_fails_when_available_reduced_by_held_funds() {
    // Deposit 100, dispute 80 → available = 20, held = 80, total = 100.
    // Withdraw 50 must fail because available (20) < 50.
    let engine = run(vec![
        Deposit::new(1.into(), 1.into(), dec!(80.0)).into(),
        Deposit::new(1.into(), 2.into(), dec!(20.0)).into(),
        Dispute::new(1.into(), 1.into()).into(),
        Withdrawal::new(1.into(), 3.into(), dec!(50.0)).into(), // must be rejected
    ]);

    let expected = HashMap::from([(ClientId::from(1), account(dec!(20.0), dec!(80.0), false))]);

    assert_eq!(engine.client_accounts().as_map(), &expected);
}
