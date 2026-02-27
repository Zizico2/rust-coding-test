## AI Usage

- I used AI autocomplete (Copilot in VS Code).
- The tests are mostly AI generated with some manual edits.
- This README is also mostly AI-generated with manual edits. Asked it to generate tests for the challenge spec and for my assumptions.

## Architecture

```
src/
├── main.rs          # CLI entry point (clap), wires CSV reader → engine → CSV writer
├── lib.rs           # Public module declarations
├── domain.rs        # Core types: Transaction variants, Account, Balance, ClientId, TransactionId
├── parsing.rs       # CSV deserialization into domain Transaction types via serde
├── output.rs        # CSV serialization of final client account state
└── engine/
    ├── mod.rs       # PaymentsEngine - stateful processor for all transaction types
    ├── types.rs     # ClientAccounts (HashMap<ClientId, Account>), DepositHistory
    └── errors.rs    # EngineError enum (AccountLocked, TransactionNotFound, etc.)
```

### Design Decisions

- **Strong typing via newtypes** - `ClientId(u16)`, `TransactionId(u32)`, and distinct structs for each transaction type (`Deposit`, `Withdrawal`, `Dispute`, `Resolve`, `Chargeback`) enforce correctness at compile time. An inner `MovementTransaction` (deposits/withdrawals with amounts) is distinguished from `DisputeTransaction` (disputes/resolves/chargebacks that reference an existing tx).
- **`rust_decimal::Decimal`** for all monetary values - avoids floating-point precision errors. Also, if the input CSV only has up to 4 decimal places, `Decimal` won't exceed that precision (subtractions and additions take the precision of the most precise operand).
- **Streaming processing** - transactions are read and processed incrementally via iterators. The full CSV is never loaded into memory at once, making the engine suitable for large inputs or integration behind a network stream.
- **Separation of concerns** - parsing, domain logic, engine orchestration, and output serialization live in distinct modules with minimal coupling.
- **Graceful error handling** - malformed CSV rows and invalid transactions are logged via `tracing` and skipped; they never crash the program. Engine-level errors (`EngineError`) are modeled with `thiserror` and converted from domain errors via `From`.

## Assumptions

1. **Only deposits can be disputed.** According to the exercise, disputes result in held funds which only makes sense for deposits.
2. **A transaction can be disputed multiple times if it is resolved.** If it is charged back, it cannot be disputed again.
3. **A locked account only disables deposits and withdrawals.**
4. **A client's first transaction should always be a deposit, in theory.** In case it isn't the account is still created with 0 balance and the transaction is ignored.
5. **Negative available balances from disputes are allowed.** If a deposit is partially withdrawn and then disputed, the available balance can go negative. This represents a debt to the disputing partner and is consistent with how real payment processors handle chargebacks.

## Safety & Error Handling

- **No `unsafe` code** - the entire codebase relies on safe Rust.
- **`thiserror`-based error types** - `DomainError` (e.g. `InsufficientFunds`) and `EngineError` (e.g. `AccountLocked`, `TransactionNotFound`, `TransactionAlreadyDisputed`) provide clear, structured error reporting.
- **Malformed input is skipped, not fatal** - CSV parsing errors and conversion failures are logged with `tracing::warn` and the row is silently discarded, so a single bad record doesn't halt processing of the remaining file.

## Correctness

Correctness is validated through **integration tests** split across focused test modules in `tests/`:

| Module | Tests |
|---|---|
| `deposit_tests` | single deposit, multiple deposits accumulate, multi-client independence, decimal precision |
| `withdrawal_tests` | basic withdrawal, exact balance, overdraft rejection, no prior deposit, fails when available reduced by held funds |
| `dispute_tests` | funds moved to held, nonexistent tx, wrong client, duplicate dispute, partial deposit, withdrawal tx ignored, multiple concurrent disputes, interleaved disputes with mixed outcomes, total = available + held invariant |
| `resolve_tests` | release held funds, no prior dispute, nonexistent tx, re-dispute after resolve, wrong client |
| `chargeback_tests` | funds removed + account locked, no prior dispute, nonexistent tx, partial balance preserved, wrong client, after resolve without re-dispute, re-dispute after chargeback |
| `locked_account_tests` | deposits ignored, withdrawals ignored, disputes allowed, resolves allowed, chargebacks allowed |
| `lifecycle_tests` | multi-client isolation, interleaved transactions, full dispute→resolve cycle, full dispute→chargeback cycle, re-dispute after resolve then chargeback |
| `io_tests` | CSV input deserialization, CSV output serialization |

Run all tests with:

```bash
cargo test
```

## Efficiency

- Transactions are **streamed** through an iterator chain - memory usage is proportional to the number of *unique clients and deposit records* (needed for dispute lookups), not to the total number of transactions.
- `DepositHistory` stores only deposits (not withdrawals) in a `HashMap<TransactionId, Deposit>`, the minimum state required for dispute resolution.
- `disputed_transactions` is tracked with a `HashSet<TransactionId>` for O(1) dispute-state lookups.
- Client accounts are stored in a `HashMap<ClientId, Account>` for O(1) access.

