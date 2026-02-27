use rust_coding_test::{
    domain::{Account, Balance},
    engine::PaymentsEngine,
};

pub fn run(transactions: Vec<rust_coding_test::domain::Transaction>) -> PaymentsEngine {
    let mut engine = PaymentsEngine::new();
    engine.process_transactions(transactions.into_iter());
    engine
}

#[allow(dead_code)]
pub fn account(
    available: rust_decimal::Decimal,
    held: rust_decimal::Decimal,
    locked: bool,
) -> Account {
    Account {
        balance: Balance::new(available, held),
        locked,
    }
}
