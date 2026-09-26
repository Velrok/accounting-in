use anyhow::Result;
use rand::RngExt;

use crate::clock;
use crate::domain::AccountId;

pub(crate) fn generate_account_id() -> Result<AccountId> {
    let nanos = clock::current_timestamp()?;
    let random: u64 = rand::rng().random();

    Ok(((nanos as u128) << 64) | random as u128)
}
