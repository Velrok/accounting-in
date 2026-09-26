use anyhow::{Context, Result};
use diesel::prelude::*;

use crate::{
    domain::{Account, Timestamp, Transfer},
    schema::{accounts, transfers},
};

pub(super) fn to_i64(value: Timestamp, field: &'static str) -> Result<i64> {
    i64::try_from(value).with_context(|| format!("{field} exceeds i64 range"))
}

fn from_i64(value: i64, field: &'static str) -> Result<Timestamp> {
    Timestamp::try_from(value).with_context(|| format!("{field} is negative"))
}

pub(super) fn to_u128_bytes(value: u128) -> Vec<u8> {
    value.to_be_bytes().to_vec()
}

pub(super) fn from_u128_bytes(bytes: Vec<u8>, field: &'static str) -> Result<u128> {
    let array: [u8; 16] = bytes
        .try_into()
        .map_err(|_| anyhow::anyhow!("{field} is not 16 bytes"))?;
    Ok(u128::from_be_bytes(array))
}

#[derive(Insertable, Queryable)]
#[diesel(table_name = transfers)]
pub(super) struct TransferRow {
    id: Vec<u8>,
    credit: Vec<u8>,
    debit: Vec<u8>,
    amount: Vec<u8>,
    bundle: i64,
    created_at: i64,
    valid_from: i64,
}

impl TryFrom<&Transfer> for TransferRow {
    type Error = anyhow::Error;

    fn try_from(transfer: &Transfer) -> Result<Self> {
        Ok(Self {
            id: to_u128_bytes(transfer.id),
            credit: to_u128_bytes(transfer.credit),
            debit: to_u128_bytes(transfer.debit),
            amount: to_u128_bytes(transfer.amount),
            bundle: to_i64(transfer.bundle, "bundle")?,
            created_at: to_i64(transfer.created_at, "created_at")?,
            valid_from: to_i64(transfer.valid_from, "valid_from")?,
        })
    }
}

#[derive(Insertable, Queryable)]
#[diesel(table_name = accounts)]
pub(super) struct AccountRow {
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
            id: to_u128_bytes(account.id),
            credits: to_u128_bytes(account.credits),
            debits: to_u128_bytes(account.debits),
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

impl TryFrom<AccountRow> for Account {
    type Error = anyhow::Error;

    fn try_from(row: AccountRow) -> Result<Self> {
        Ok(Self {
            id: from_u128_bytes(row.id, "id")?,
            credits: from_u128_bytes(row.credits, "credits")?,
            debits: from_u128_bytes(row.debits, "debits")?,
            ledger: row.ledger as u32,
            code: row.code as u32,
            created_at: from_i64(row.created_at, "created_at")?,
            deprecated_at: row
                .deprecated_at
                .map(|t| from_i64(t, "deprecated_at"))
                .transpose()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn u128_bytes_round_trip() {
        let value = u128::MAX - 1;

        let bytes = to_u128_bytes(value);
        let restored = from_u128_bytes(bytes, "value").unwrap();

        assert_eq!(restored, value);
    }

    #[test]
    fn from_u128_bytes_rejects_wrong_length() {
        let result = from_u128_bytes(vec![1, 2, 3], "value");

        assert!(result.is_err());
    }

    #[test]
    fn i64_round_trip() {
        let value: Timestamp = 123;

        let encoded = to_i64(value, "value").unwrap();
        let restored = from_i64(encoded, "value").unwrap();

        assert_eq!(restored, value);
    }

    #[test]
    fn to_i64_rejects_values_exceeding_i64_range() {
        let value = Timestamp::MAX;

        let result = to_i64(value, "value");

        assert!(result.is_err());
    }

    #[test]
    fn from_i64_rejects_negative_values() {
        let result = from_i64(-1, "value");

        assert!(result.is_err());
    }

    #[test]
    fn account_row_round_trip() {
        let mut account = Account::new(42, 700, 3, 100);
        account.credits = 100;
        account.debits = 50;
        account.deprecated_at = Some(999);

        let row = AccountRow::try_from(&account).unwrap();
        let restored = Account::try_from(row).unwrap();

        assert_eq!(restored.id, account.id);
        assert_eq!(restored.credits, account.credits);
        assert_eq!(restored.debits, account.debits);
        assert_eq!(restored.ledger, account.ledger);
        assert_eq!(restored.code, account.code);
        assert_eq!(restored.created_at, account.created_at);
        assert_eq!(restored.deprecated_at, account.deprecated_at);
    }
}
