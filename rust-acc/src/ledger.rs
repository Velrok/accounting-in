use anyhow::Result;
use diesel::SqliteConnection;

use crate::{domain::Account, storage};

pub fn all_accounts() -> Vec<Account> {
    vec![]
}

pub fn commit_account(conn: &mut SqliteConnection, account: &Account) -> Result<()> {
    storage::save_account(conn, account)
}
