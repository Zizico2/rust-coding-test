use crate::domain::{Account, ClientId, Deposit, DisputeState, TransactionId};
use std::collections::HashMap;

/// Stores all successfully processed deposits, keyed by transaction ID.
/// Only deposits are stored because they're the only transaction type that can be disputed.
#[derive(Debug)]
pub struct DepositHistory(HashMap<TransactionId, Deposit>);

impl Default for DepositHistory {
    fn default() -> Self {
        Self::new()
    }
}

impl DepositHistory {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn add_deposit(&mut self, deposit: Deposit) {
        self.0.insert(deposit.transaction_id(), deposit);
    }
    /// Looks up a deposit by tx ID, but only returns it if it belongs to the given client.
    /// This prevents a client from disputing another client's deposit.
    pub fn get_deposit(&self, tx_id: &TransactionId, client_id: &ClientId) -> Option<&Deposit> {
        self.0.get(tx_id).filter(|tx| &tx.client_id() == client_id)
    }
    pub fn get_deposit_under_dispute_mut(&mut self, tx_id: &TransactionId) -> Option<&mut Deposit> {
        self.0
            .get_mut(tx_id)
            .filter(|tx| tx.dispute == DisputeState::Open)
    }
    pub fn get_deposit_undisputed(&self, tx_id: &TransactionId) -> Option<&Deposit> {
        self.0
            .get(tx_id)
            .filter(|tx| tx.dispute == DisputeState::None)
    }
}

/// Maps each client to their account. Accounts are lazily created on first transaction.
#[derive(Debug)]
pub struct ClientAccounts(HashMap<ClientId, Account>);

impl Default for ClientAccounts {
    fn default() -> Self {
        Self::new()
    }
}

impl ClientAccounts {
    pub fn new() -> Self {
        Self(HashMap::new())
    }
    pub fn as_map(&self) -> &HashMap<ClientId, Account> {
        &self.0
    }
    pub fn get_or_create_account_mut(&mut self, client_id: ClientId) -> &mut Account {
        self.0.entry(client_id).or_default()
    }

    pub fn get_or_create_account(&mut self, client_id: ClientId) -> &Account {
        self.0.entry(client_id).or_default()
    }
}
