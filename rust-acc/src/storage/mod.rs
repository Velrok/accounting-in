mod conversions;

use anyhow::Result;
use diesel::prelude::*;

use crate::{
    domain::{Account, AccountId, Timestamp},
    ledger::TransfersBundle,
    schema::{accounts, transfers},
};
use conversions::{AccountRow, TransferRow, to_i64, to_u128_bytes};

pub(crate) fn establish(database_url: &str) -> Result<diesel::SqliteConnection> {
    let mut conn = diesel::SqliteConnection::establish(database_url)?;
    diesel::sql_query("PRAGMA foreign_keys = ON").execute(&mut conn)?;
    Ok(conn)
}

pub(crate) fn save_account(conn: &mut diesel::SqliteConnection, account: &Account) -> Result<()> {
    let new_account = AccountRow::try_from(account)?;

    diesel::insert_into(accounts::table)
        .values(&new_account)
        .execute(conn)?;

    Ok(())
}

pub(crate) fn list_accounts(conn: &mut diesel::SqliteConnection) -> Result<Vec<Account>> {
    accounts::table
        .load::<AccountRow>(conn)?
        .into_iter()
        .map(Account::try_from)
        .collect()
}

pub(crate) fn deprecate_account(
    conn: &mut diesel::SqliteConnection,
    id: AccountId,
    deprecated_at: Timestamp,
) -> Result<()> {
    let deprecated_at = to_i64(deprecated_at, "deprecated_at")?;

    let rows = diesel::update(accounts::table.filter(accounts::id.eq(to_u128_bytes(id))))
        .set(accounts::deprecated_at.eq(deprecated_at))
        .execute(conn)?;

    if rows == 0 {
        anyhow::bail!("account {id} not found");
    }

    Ok(())
}

pub(crate) fn commit_transaction_bundle(
    conn: &mut diesel::SqliteConnection,
    bundle: &TransfersBundle,
) -> Result<()> {
    let rows = bundle
        .transactions
        .iter()
        .map(TransferRow::try_from)
        .collect::<Result<Vec<_>>>()?;

    diesel::insert_into(transfers::table)
        .values(&rows)
        .execute(conn)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};

    use super::*;
    use crate::domain::Transfer;

    const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

    fn test_conn() -> SqliteConnection {
        let mut conn = establish(":memory:").unwrap();
        conn.run_pending_migrations(MIGRATIONS).unwrap();
        conn
    }

    #[test]
    fn saves_account() {
        let mut conn = test_conn();
        let account = Account::new(1, 700, 1, 100);

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
        let account = Account::new(1, 700, 1, 100);

        save_account(&mut conn, &account).unwrap();
        let result = save_account(&mut conn, &account);

        assert!(result.is_err());
    }

    #[test]
    fn rejects_duplicate_ledger_and_code() {
        let mut conn = test_conn();

        save_account(&mut conn, &Account::new(1, 700, 1, 100)).unwrap();
        let result = save_account(&mut conn, &Account::new(2, 700, 1, 100));

        assert!(result.is_err());
    }

    #[test]
    fn lists_saved_accounts() {
        let mut conn = test_conn();
        save_account(&mut conn, &Account::new(1, 700, 1, 100)).unwrap();
        save_account(&mut conn, &Account::new(2, 700, 2, 100)).unwrap();

        let mut accounts = list_accounts(&mut conn).unwrap();
        accounts.sort_by_key(|account| account.id);

        assert_eq!(accounts.len(), 2);
        assert_eq!(accounts[0].id, 1);
        assert_eq!(accounts[1].id, 2);
    }

    #[test]
    fn deprecates_existing_account() {
        let mut conn = test_conn();
        save_account(&mut conn, &Account::new(1, 700, 1, 100)).unwrap();

        deprecate_account(&mut conn, 1, 123).unwrap();

        let accounts = list_accounts(&mut conn).unwrap();
        assert_eq!(accounts[0].deprecated_at, Some(123));
    }

    #[test]
    fn rejects_deprecating_unknown_account() {
        let mut conn = test_conn();

        let result = deprecate_account(&mut conn, 1, 123);

        assert!(result.is_err());
    }

    fn transfer(id: u128, credit: u128, debit: u128, amount: u128, bundle: u64) -> Transfer {
        Transfer {
            id,
            credit,
            debit,
            amount,
            bundle,
            created_at: 100,
            valid_from: 100,
        }
    }

    #[test]
    fn commits_transaction_bundle() {
        let mut conn = test_conn();
        save_account(&mut conn, &Account::new(1, 700, 1, 100)).unwrap();
        save_account(&mut conn, &Account::new(2, 700, 2, 100)).unwrap();
        let bundle = TransfersBundle {
            transactions: vec![transfer(10, 1, 2, 50, 999)],
        };

        commit_transaction_bundle(&mut conn, &bundle).unwrap();

        let (credit, debit) = transfers::table
            .select((transfers::credit, transfers::debit))
            .filter(transfers::id.eq(to_u128_bytes(10)))
            .first::<(Vec<u8>, Vec<u8>)>(&mut conn)
            .unwrap();

        assert_eq!(credit, to_u128_bytes(1));
        assert_eq!(debit, to_u128_bytes(2));
    }

    #[test]
    fn rejects_transfer_referencing_unknown_account() {
        let mut conn = test_conn();
        save_account(&mut conn, &Account::new(1, 700, 1, 100)).unwrap();
        let bundle = TransfersBundle {
            transactions: vec![transfer(10, 1, 2, 50, 999)],
        };

        let result = commit_transaction_bundle(&mut conn, &bundle);

        assert!(result.is_err());
    }

    #[test]
    fn commit_is_all_or_nothing() {
        let mut conn = test_conn();
        save_account(&mut conn, &Account::new(1, 700, 1, 100)).unwrap();
        save_account(&mut conn, &Account::new(2, 700, 2, 100)).unwrap();
        let bundle = TransfersBundle {
            transactions: vec![transfer(10, 1, 2, 50, 999), transfer(11, 1, 3, 50, 999)],
        };

        let result = commit_transaction_bundle(&mut conn, &bundle);

        assert!(result.is_err());
        let count: i64 = transfers::table.count().get_result(&mut conn).unwrap();
        assert_eq!(count, 0);
    }
}
