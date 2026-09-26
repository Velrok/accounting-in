use anyhow::Result;
use rand::RngExt;

use crate::clock;
use crate::domain::{AccountId, BundleId, TransferId};

fn timestamp_random_id() -> Result<u128> {
    let nanos = clock::current_timestamp()?;
    let random: u64 = rand::rng().random();

    Ok(((nanos as u128) << 64) | random as u128)
}

pub(crate) fn generate_account_id() -> Result<AccountId> {
    timestamp_random_id()
}

pub(crate) fn generate_transfer_id() -> Result<TransferId> {
    timestamp_random_id()
}

pub(crate) fn generate_bundle_id() -> BundleId {
    rand::rng().random()
}
