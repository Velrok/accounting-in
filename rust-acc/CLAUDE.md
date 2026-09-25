We are building out a minimal accounting service inspired by TigerBeetle.

## Storage

- Sync Diesel + SQLite (not async/sqlx) — avoids async leaking through the app.
- `u128` fields stored as 16-byte big-endian `BLOB` (preserves ordering under SQLite's byte-wise comparison; `TEXT` would sort wrong and cost more space).
- Timestamps are nanosecond epoch `u64` stored as `BIGINT`/`i64` (safe until year 2262).
- `diesel_cli` is a global install, not a project dep: `cargo install diesel_cli --no-default-features --features sqlite`; regenerate `src/schema.rs` via `diesel print-schema` after migration changes.

## Domain semantics

- Accounts are insert-only/immutable: no upsert, duplicate `id` fails (UNIQUE constraint).
- Transfers must reference existing accounts — no auto-vivification.
