use anyhow::{Context, Result};
use diesel::prelude::*;

use crate::{domain::Account, schema::accounts};

fn to_i64(value: u64, field: &'static str) -> Result<i64> {
    i64::try_from(value).with_context(|| format!("{field} exceeds i64 range"))
}

#[derive(Insertable)]
#[diesel(table_name = accounts)]
struct AccountRow {
    id: Vec<u8>,
    credits: Vec<u8>,
    debits: Vec<u8>,
    ledger: i32,
    code: i32,
    created_at: i64,
    deprecated_at: Option<i64>,
}

impl TryFrom<&Account> for AccountRow {
    type Error = anyhow::Error;

    fn try_from(account: &Account) -> Result<Self> {
        Ok(Self {
            id: account.id.to_be_bytes().to_vec(),
            credits: account.credits.to_be_bytes().to_vec(),
            debits: account.debits.to_be_bytes().to_vec(),
            ledger: account.ledger as i32,
            code: account.code as i32,
            created_at: to_i64(account.created_at, "created_at")?,
            deprecated_at: account
                .deprecated_at
                .map(|t| to_i64(t, "deprecated_at"))
                .transpose()?,
        })
    }
}

pub(crate) fn save_account(conn: &mut diesel::SqliteConnection, account: &Account) -> Result<()> {
    let new_account = AccountRow::try_from(account)?;

    diesel::insert_into(accounts::table)
        .values(&new_account)
        .execute(conn)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};

    use super::*;

    const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

    fn test_conn() -> SqliteConnection {
        let mut conn = SqliteConnection::establish(":memory:").unwrap();
        conn.run_pending_migrations(MIGRATIONS).unwrap();
        conn
    }

    #[test]
    fn saves_account() {
        let mut conn = test_conn();
        let account = Account::new(1, 700, 1);

        save_account(&mut conn, &account).unwrap();

        let (credits, debits) = accounts::table
            .select((accounts::credits, accounts::debits))
            .filter(accounts::id.eq(1u128.to_be_bytes().to_vec()))
            .first::<(Vec<u8>, Vec<u8>)>(&mut conn)
            .unwrap();

        assert_eq!(credits, 0u128.to_be_bytes().to_vec());
        assert_eq!(debits, 0u128.to_be_bytes().to_vec());
    }

    #[test]
    fn rejects_duplicate_id() {
        let mut conn = test_conn();
        let account = Account::new(1, 700, 1);

        save_account(&mut conn, &account).unwrap();
        let result = save_account(&mut conn, &account);

        assert!(result.is_err());
    }
}
