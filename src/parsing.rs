//! CSV deserialization.
//!
//! Parsing happens in two stages:
//! 1. Serde deserializes each CSV row into a flat `CsvTransaction`.
//! 2. `TryFrom<CsvTransaction>` converts it into the strongly-typed domain `Transaction`.
//!
//! Malformed rows or missing required fields are logged and skipped.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use tracing::warn;

use crate::domain::{
    Chargeback, ClientId, Deposit, Dispute, Resolve, Transaction, TransactionId, Withdrawal,
};

#[derive(Debug, Clone, Copy, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum TransactionType {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}

/// Flat representation of a single CSV row. `amount` is optional because
/// dispute/resolve/chargeback rows don't carry one.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CsvTransaction {
    r#type: TransactionType,
    client: ClientId,
    tx: TransactionId,
    amount: Option<Decimal>,
}

/// Returns an iterator that lazily deserializes CSV rows into domain transactions,
/// skipping any rows that fail to parse or convert.
pub fn deserialize_csv<D: std::io::Read>(
    reader: &mut csv::Reader<D>,
) -> impl Iterator<Item = Transaction> {
    let transaction_iter = reader.deserialize::<CsvTransaction>();

    transaction_iter
        .filter_map(|result| match result {
            Ok(transaction) => Some(transaction),
            Err(e) => {
                // skipping malformed transaction and logging the error
                warn!("Failed to parse transaction: {e}");
                None
            }
        })
        .filter_map(
            |csv_transaction| match Transaction::try_from(csv_transaction) {
                Ok(transaction) => Some(transaction),
                Err(e) => {
                    // skipping transaction that failed to convert and logging the error
                    warn!("Failed to convert CsvTransaction to Transaction: {e}");
                    None
                }
            },
        )
}

#[derive(Debug, thiserror::Error)]
enum IntoTransactionError {
    #[error("Missing amount for deposit")]
    MissingAmountForDeposit,
    #[error("Missing amount for withdrawal")]
    MissingAmountForWithdrawal,
}

impl TryFrom<CsvTransaction> for Transaction {
    type Error = IntoTransactionError;

    fn try_from(value: CsvTransaction) -> Result<Self, Self::Error> {
        match value.r#type {
            TransactionType::Deposit => Ok(Transaction::Deposit(Deposit::new(
                value.client,
                value.tx,
                value
                    .amount
                    .ok_or(IntoTransactionError::MissingAmountForDeposit)?,
            ))),
            TransactionType::Withdrawal => Ok(Transaction::Withdrawal(Withdrawal::new(
                value.client,
                value.tx,
                value
                    .amount
                    .ok_or(IntoTransactionError::MissingAmountForWithdrawal)?,
            ))),
            TransactionType::Dispute => {
                Ok(Transaction::Dispute(Dispute::new(value.client, value.tx)))
            }
            TransactionType::Resolve => {
                Ok(Transaction::Resolve(Resolve::new(value.client, value.tx)))
            }
            TransactionType::Chargeback => Ok(Transaction::Chargeback(Chargeback::new(
                value.client,
                value.tx,
            ))),
        }
    }
}
