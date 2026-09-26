mod clock;
mod domain;
mod id_generators;
mod ledger;
mod schema;
mod storage;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};

use domain::{Account, AccountId, GroupingId};
use id_generators::generate_account_id;

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

fn main() -> Result<()> {
    let cli = Cli::parse();
    let database_url = std::env::var("DATABASE_URL").context("DATABASE_URL must be set")?;
    let mut conn = storage::establish(&database_url)?;

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
                let created_at = clock::current_timestamp()?;
                let account = Account::new(id, ledger, code, created_at);
                storage::save_account(&mut conn, &account)?;
                println!("created account {id}");
            }
            AccountsCommand::Deprecate { account_id } => {
                let now = clock::current_timestamp()?;
                storage::deprecate_account(&mut conn, account_id, now)?;
                println!("deprecated account {account_id}");
            }
        },
    }

    Ok(())
}
