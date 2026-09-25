mod domain;
mod ledger;
mod schema;
mod storage;

use anyhow::Result;
use clap::{Parser, Subcommand};
use diesel::prelude::*;
use rand::RngExt;

use domain::{Account, AccountId, GroupingId};

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Manage accounts
    Accounts {
        #[command(subcommand)]
        command: AccountsCommand,
    },
}

#[derive(Subcommand)]
enum AccountsCommand {
    /// List all accounts
    List,
    /// Create a new account
    Create {
        #[arg(long)]
        ledger: GroupingId,
        #[arg(long)]
        code: GroupingId,
    },
    /// Deprecate an existing account
    Deprecate { account_id: AccountId },
}

fn generate_account_id() -> Result<AccountId> {
    let nanos = domain::current_timestamp()?;
    let random: u64 = rand::rng().random();

    Ok(((nanos as u128) << 64) | random as u128)
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let mut conn = SqliteConnection::establish("accounting.db")?;

    match cli.command {
        Command::Accounts { command } => match command {
            AccountsCommand::List => {
                for account in storage::list_accounts(&mut conn)? {
                    println!(
                        "{}\tledger={}\tcode={}\tcredits={}\tdebits={}\tcreated_at={}\tdeprecated_at={}",
                        account.id,
                        account.ledger,
                        account.code,
                        account.credits,
                        account.debits,
                        account.created_at,
                        account
                            .deprecated_at
                            .map(|t| t.to_string())
                            .unwrap_or_else(|| "-".to_string()),
                    );
                }
            }
            AccountsCommand::Create { ledger, code } => {
                let id = generate_account_id()?;
                let account = Account::new(id, ledger, code);
                storage::save_account(&mut conn, &account)?;
                println!("created account {id}");
            }
            AccountsCommand::Deprecate { account_id } => {
                let now = domain::current_timestamp()?;
                storage::deprecate_account(&mut conn, account_id, now)?;
                println!("deprecated account {account_id}");
            }
        },
    }

    Ok(())
}
