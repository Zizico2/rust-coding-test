use crate::domain::DomainError;

#[derive(Debug, thiserror::Error)]
pub enum EngineError {
    #[error("Account is locked")]
    AccountLocked,
    #[error("Transaction not found")]
    TransactionNotFound,
    #[error("Transaction already disputed")]
    TransactionAlreadyDisputed,
    #[error("Transaction not disputed")]
    TransactionNotDisputed,
    #[error("Domain error: {0}")]
    DomainError(#[from] DomainError),
}
