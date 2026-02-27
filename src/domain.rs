//! Core domain types: transactions, accounts, and balances.

use derive_more::{From, Into, TryInto};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Newtype wrapper for client identifiers (valid u16 per spec).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, From, Into)]
pub struct ClientId(u16);

/// Newtype wrapper for globally-unique transaction identifiers (valid u32 per spec).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, From, Into)]
pub struct TransactionId(u32);

#[derive(Debug, thiserror::Error)]
pub enum DomainError {
    #[error("Insufficient funds")]
    InsufficientFunds,
}

/// Sum type over all transaction kinds the engine can process.
#[derive(Debug, From, TryInto, PartialEq)]
pub enum Transaction {
    Deposit(Deposit),
    Withdrawal(Withdrawal),
    Dispute(Dispute),
    Resolve(Resolve),
    Chargeback(Chargeback),
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum DisputeState {
    /// No dispute is open for this transaction.
    None,
    /// A dispute is currently open for this transaction.
    Open,
    /// A dispute was open but has now been charged back.
    ChargedBack,
}
// Movement transactions carry an amount (deposits & withdrawals).
#[derive(Debug, PartialEq)]
pub struct Deposit {
    pub dispute: DisputeState,
    tx: MovementTransaction,
}
#[derive(Debug, PartialEq)]
pub struct Withdrawal(MovementTransaction);

// Dispute-family transactions reference an existing tx by ID (no amount field).
#[derive(Debug, PartialEq)]
pub struct Dispute(DisputeTransaction);
#[derive(Debug, PartialEq)]
pub struct Resolve(DisputeTransaction);
#[derive(Debug, PartialEq, From)]
pub struct Chargeback(DisputeTransaction);

impl Deposit {
    pub fn new(client: ClientId, tx: TransactionId, amount: Decimal) -> Self {
        Self {
            tx: MovementTransaction::new(client, tx, amount),
            dispute: DisputeState::None,
        }
    }
    pub fn amount(&self) -> Decimal {
        self.tx.amount
    }
    pub fn client_id(&self) -> ClientId {
        self.tx.client
    }
    pub fn transaction_id(&self) -> TransactionId {
        self.tx.tx
    }
}

impl Withdrawal {
    pub fn new(client: ClientId, tx: TransactionId, amount: Decimal) -> Self {
        Self(MovementTransaction::new(client, tx, amount))
    }
    pub fn amount(&self) -> Decimal {
        self.0.amount
    }
    pub fn client_id(&self) -> ClientId {
        self.0.client
    }

    pub fn transaction_id(&self) -> TransactionId {
        self.0.tx
    }
}

impl Dispute {
    pub fn new(client: ClientId, disputed_tx: TransactionId) -> Self {
        Self(DisputeTransaction::new(client, disputed_tx))
    }
    pub fn client_id(&self) -> ClientId {
        self.0.client_id()
    }
    pub fn disputed_tx_id(&self) -> TransactionId {
        self.0.disputed_transaction_id()
    }
}

impl Resolve {
    pub fn new(client: ClientId, disputed_tx: TransactionId) -> Self {
        Self(DisputeTransaction::new(client, disputed_tx))
    }
    pub fn client_id(&self) -> ClientId {
        self.0.client_id()
    }
    pub fn disputed_tx_id(&self) -> TransactionId {
        self.0.disputed_transaction_id()
    }
}
impl Chargeback {
    pub fn new(client: ClientId, disputed_tx: TransactionId) -> Self {
        Self(DisputeTransaction::new(client, disputed_tx))
    }
    pub fn client_id(&self) -> ClientId {
        self.0.client_id()
    }
    pub fn disputed_tx_id(&self) -> TransactionId {
        self.0.disputed_transaction_id()
    }
}

/// A single client account. Locked accounts reject all further operations.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Account {
    pub balance: Balance,
    pub locked: bool,
}

/// Tracks a client's funds. Invariant: total = available + held.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Balance {
    available: Decimal,
    held: Decimal,
}

impl Balance {
    pub fn new(available: Decimal, held: Decimal) -> Self {
        Self { available, held }
    }
    pub fn available(&self) -> Decimal {
        self.available
    }
    pub fn held(&self) -> Decimal {
        self.held
    }
    pub fn total(&self) -> Decimal {
        self.available + self.held
    }
    /// Credit funds (deposit). Increases available.
    pub fn add(&mut self, amount: Decimal) {
        self.available += amount;
    }
    /// Move funds from available to held (dispute). Total stays the same.
    pub fn hold(&mut self, amount: Decimal) {
        self.available -= amount;
        self.held += amount;
    }
    /// Move funds from held to available (resolve). Total stays the same.
    pub fn release(&mut self, amount: Decimal) {
        self.held -= amount;
        self.available += amount;
    }
    /// Debit funds (withdrawal). Fails if available < amount.
    pub fn try_remove(&mut self, amount: Decimal) -> Result<(), DomainError> {
        if self.available >= amount {
            self.available -= amount;
        } else {
            return Err(DomainError::InsufficientFunds);
        }
        Ok(())
    }
    pub fn remove(&mut self, amount: Decimal) {
        self.available -= amount;
    }
}

/// Inner struct shared by Deposit and Withdrawal - transactions that carry an amount.
#[derive(Debug, PartialEq)]
struct MovementTransaction {
    client: ClientId,
    tx: TransactionId,
    amount: Decimal,
}
impl MovementTransaction {
    pub fn new(client: ClientId, tx: TransactionId, amount: Decimal) -> Self {
        Self { client, tx, amount }
    }
}

/// Inner struct shared by Dispute, Resolve, and Chargeback - they reference an existing tx.
#[derive(Debug, PartialEq)]
struct DisputeTransaction {
    client: ClientId,
    disputed_tx: TransactionId,
}

impl DisputeTransaction {
    pub fn new(client: ClientId, disputed_tx: TransactionId) -> Self {
        Self {
            client,
            disputed_tx,
        }
    }
    pub fn client_id(&self) -> ClientId {
        self.client
    }
    pub fn disputed_transaction_id(&self) -> TransactionId {
        self.disputed_tx
    }
}
