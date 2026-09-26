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
    // Stored as a BIGINT (i64), so keep the top bit clear.
    let random: u64 = rand::rng().random();
    random >> 1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_bundle_ids_fit_in_i64_storage() {
        for _ in 0..1000 {
            let id = generate_bundle_id();
            assert!(i64::try_from(id).is_ok(), "bundle id {id} exceeds i64 range");
        }
    }
}
