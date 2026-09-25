mod domain;
mod ledger;
mod schema;
mod storage;

use diesel::prelude::*;

fn main() -> anyhow::Result<()> {
    let mut conn = SqliteConnection::establish("accounting.db")?;

    Ok(())
}
