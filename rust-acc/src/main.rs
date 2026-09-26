mod clock;
mod domain;
mod id_generators;
mod ledger;
mod schema;
mod storage;
mod gbp;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};

use domain::{Account, AccountId, Amount, GroupingId, LedgerId, Timestamp};
use gbp::GBP;
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
    /// Manage transfers
    Transfers {
        #[command(subcommand)]
        command: TransfersCommand,
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

#[derive(Subcommand)]
enum TransfersCommand {
    /// Create a new transfer
    Create {
        #[arg(long)]
        credit: AccountId,
        #[arg(long)]
        debit: AccountId,
        #[arg(long)]
        amount: f64,
        #[arg(long)]
        valid_from: Option<Timestamp>,
    },
    /// List transfers for a ledger
    List {
        #[arg(long)]
        ledger: LedgerId,
    },
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
                        "{}\tledger={}\tcode={}\tbalance={}\tcreated_at={}\tdeprecated_at={}",
                        account.id,
                        account.ledger,
                        account.code,
                        GBP::from(&account),
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
        Command::Transfers { command } => match command {
            TransfersCommand::Create {
                credit,
                debit,
                amount,
                valid_from,
            } => {
                let credit_account = storage::get_account(&mut conn, credit)?;
                let debit_account = storage::get_account(&mut conn, debit)?;
                let amount: Amount = GBP::new(amount)
                    .as_raw()
                    .try_into()
                    .context("amount must be positive")?;
                let request = ledger::TransferRequest::new(credit_account, debit_account, amount)?;
                let valid_from = match valid_from {
                    Some(valid_from) => valid_from,
                    None => clock::current_timestamp()?,
                };
                let bundle = ledger::bundle_transfers(&[request], valid_from)?;
                let bundle_id = bundle.transactions[0].bundle;
                storage::commit_transaction_bundle(&mut conn, &bundle)?;
                println!("committed transfer bundle {bundle_id}");
            }
            TransfersCommand::List { ledger } => {
                for transfer in storage::list_transfers_for_ledger(&mut conn, ledger)? {
                    println!(
                        "{}\tcredit={}\tdebit={}\tamount={}\tbundle={}\tcreated_at={}\tvalid_from={}",
                        transfer.id,
                        transfer.credit,
                        transfer.debit,
                        transfer.amount,
                        transfer.bundle,
                        transfer.created_at,
                        transfer.valid_from,
                    );
                }
            }
        },
    }

    Ok(())
}
