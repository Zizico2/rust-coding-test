//! Serializes final account state to CSV.

use rust_decimal::Decimal;
use serde::Serialize;

use crate::{domain::ClientId, engine::ClientAccounts};

/// Maps directly to the required output columns: client, available, held, total, locked.
#[derive(Debug, Serialize)]
struct OutputCsv {
    client: ClientId,
    available: Decimal,
    held: Decimal,
    total: Decimal,
    locked: bool,
}

pub fn print_accounts(
    client_accounts: &ClientAccounts,
    writer: impl std::io::Write,
) -> anyhow::Result<()> {
    let mut wtr = csv::Writer::from_writer(writer);
    for (client_id, account) in client_accounts.as_map() {
        let output_csv = OutputCsv {
            client: *client_id,
            available: account.balance.available(),
            held: account.balance.held(),
            total: account.balance.total(),
            locked: account.locked,
        };
        wtr.serialize(output_csv)?;
    }
    wtr.flush()?;
    Ok(())
}
