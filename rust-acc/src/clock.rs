use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};

use crate::domain::Timestamp;

pub(crate) fn current_timestamp() -> Result<Timestamp> {
    let nanos: u64 = SystemTime::now()
        .duration_since(UNIX_EPOCH)?
        .as_nanos()
        .try_into()
        .context("System time exceeds u64 nanosec range")?;
    Ok(nanos)
}
