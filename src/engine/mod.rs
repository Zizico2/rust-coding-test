//! Stateful payments engine.
//!
//! Processes a stream of transactions and maintains per-client account balances,
//! a history of deposits (needed for dispute lookups), and a set of currently
//! disputed transaction IDs.

use tracing::warn;

use crate::{
    domain::{
        Account, Chargeback, Deposit, Dispute, DisputeState, Resolve, Transaction, Withdrawal,
    },
    engine::errors::EngineError,
};
pub use types::{ClientAccounts, DepositHistory};

pub mod errors;
mod types;

pub struct PaymentsEngine {
    client_accounts: ClientAccounts,
    /// Only deposits are stored - they're the only transaction type that can be disputed.
    deposit_history: DepositHistory,
    // /// Tracks which transaction IDs are currently under dispute.
    // disputed_transactions: HashSet<TransactionId>,
}

impl PaymentsEngine {
    pub fn client_accounts(&self) -> &ClientAccounts {
        &self.client_accounts
    }
}

/// Guard: all operations are rejected on a locked (frozen) account.
fn check_account_eligibility(account: &Account) -> Result<(), EngineError> {
    if account.locked {
        return Err(EngineError::AccountLocked);
    }
    Ok(())
}

impl Default for PaymentsEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl PaymentsEngine {
    pub fn new() -> Self {
        Self {
            client_accounts: ClientAccounts::new(),
            deposit_history: DepositHistory::new(),
        }
    }
    fn process_transaction(&mut self, transaction: Transaction) -> Result<(), EngineError> {
        match transaction {
            Transaction::Deposit(deposit) => self.process_deposit_transaction(deposit)?,
            Transaction::Withdrawal(withdrawal) => {
                self.process_withdrawal_transaction(withdrawal)?
            }

            Transaction::Dispute(dispute) => self.process_dispute_transaction(dispute)?,
            Transaction::Resolve(resolve) => self.process_resolve_transaction(resolve)?,
            Transaction::Chargeback(chargeback) => {
                self.process_chargeback_transaction(chargeback)?
            }
        }

        Ok(())
    }

    fn process_withdrawal_transaction(
        &mut self,
        transaction: Withdrawal,
    ) -> Result<(), EngineError> {
        let account = self
            .client_accounts
            .get_or_create_account_mut(transaction.client_id());
        check_account_eligibility(account)?;

        let amount = transaction.amount();

        account.balance.remove(amount)?;

        Ok(())
    }
    fn process_deposit_transaction(&mut self, transaction: Deposit) -> Result<(), EngineError> {
        let account = self
            .client_accounts
            .get_or_create_account_mut(transaction.client_id());
        check_account_eligibility(account)?;

        account.balance.add(transaction.amount());
        // Record the deposit so it can be referenced later by disputes.
        self.deposit_history.add_deposit(transaction);

        Ok(())
    }
    fn process_dispute_transaction(&mut self, transaction: Dispute) -> Result<(), EngineError> {
        let account = self
            .client_accounts
            .get_or_create_account_mut(transaction.client_id());

        let disputed_tx = self
            .deposit_history
            .try_get_deposit_undisputed_mut(&transaction.disputed_tx_id(), &transaction.client_id())?;

        account.balance.hold(disputed_tx.amount());
        disputed_tx.dispute = DisputeState::Open;
        Ok(())
    }
    fn process_resolve_transaction(&mut self, transaction: Resolve) -> Result<(), EngineError> {
        let account = self
            .client_accounts
            .get_or_create_account_mut(transaction.client_id());

        let disputed_tx = self
            .deposit_history
            .try_get_deposit_under_dispute_mut(&transaction.disputed_tx_id(), &transaction.client_id())?;

        account.balance.release(disputed_tx.amount());

        disputed_tx.dispute = DisputeState::None;

        Ok(())
    }
    fn process_chargeback_transaction(
        &mut self,
        transaction: Chargeback,
    ) -> Result<(), EngineError> {
        let account = self
            .client_accounts
            .get_or_create_account_mut(transaction.client_id());

        // let disputed_tx = self
        //     .deposit_history
        //     .get_deposit(&transaction.disputed_tx_id(), &transaction.client_id());
        // let Some(disputed_tx) = disputed_tx else {
        //     return Err(EngineError::TransactionNotFound);
        // };
        // if !self
        //     .disputed_transactions
        //     .contains(&disputed_tx.transaction_id())
        // {
        //     return Err(EngineError::TransactionNotDisputed);
        // }

        let disputed_tx = self
            .deposit_history
            .try_get_deposit_under_dispute_mut(&transaction.disputed_tx_id(), &transaction.client_id())?;

        account.balance.release(disputed_tx.amount());
        account.balance.remove(disputed_tx.amount())?;
        account.locked = true;

        disputed_tx.dispute = DisputeState::ChargedBack;
        // self.disputed_transactions
        //     .remove(&disputed_tx.transaction_id());
        Ok(())
    }

    pub fn process_transactions(&mut self, transactions: impl Iterator<Item = Transaction>) {
        for transaction in transactions {
            if let Err(e) = self.process_transaction(transaction) {
                warn!("Error processing transaction: {e}");
            }
        }
    }
}
