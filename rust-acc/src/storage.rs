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
