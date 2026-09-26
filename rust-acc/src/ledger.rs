use crate::{
    clock,
    domain::{Account, Amount, Timestamp, Transfer, TransferId},
    id_generators,
};

pub struct TransfersBundle {
    pub(crate) transactions: Vec<Transfer>,
}

pub struct TransferRequest {
    id: TransferId,
    credit: Account,
    debit: Account,
    amount: Amount,
}

impl TransferRequest {
    pub fn new(credit: Account, debit: Account, amount: Amount) -> anyhow::Result<Self> {
        Ok(Self {
            id: id_generators::generate_transfer_id()?,
            credit,
            debit,
            amount,
        })
    }
}

pub fn bundle_transfers(
    transfer_requests: &[TransferRequest],
    valid_from: Timestamp,
) -> anyhow::Result<TransfersBundle> {
    let Some(first) = transfer_requests.first() else {
        anyhow::bail!("transfer_requests must have at least one entry");
    };
    let ledger = Some(first.credit.ledger);
    for tr in transfer_requests {
        if Some(tr.credit.ledger) != ledger || Some(tr.debit.ledger) != ledger {
            anyhow::bail!("all accounts in a transfer bundle must belong to the same ledger");
        }
    }

    let created_at = clock::current_timestamp()?;
    let bundle = id_generators::generate_bundle_id();
    let transactions: Vec<_> = transfer_requests
        .iter()
        .map(|tr| Transfer {
            id: tr.id,
            credit: tr.credit.id,
            debit: tr.debit.id,
            amount: tr.amount,
            bundle,
            created_at,
            valid_from,
        })
        .collect();
    Ok(TransfersBundle { transactions })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn account(id: u128, ledger: u32) -> Account {
        Account::new(id, ledger, 1, 0)
    }

    #[test]
    fn new_builds_a_request_carrying_the_given_legs_and_amount() {
        let request = TransferRequest::new(account(1, 700), account(2, 700), 50).unwrap();

        let bundle = bundle_transfers(&[request], 0).unwrap();

        assert_eq!(bundle.transactions[0].credit, 1);
        assert_eq!(bundle.transactions[0].debit, 2);
        assert_eq!(bundle.transactions[0].amount, 50);
    }

    #[test]
    fn rejects_empty_requests() {
        let result = bundle_transfers(&[], 0);

        assert!(result.is_err());
    }

    #[test]
    fn rejects_mismatched_ledgers() {
        let requests = vec![
            TransferRequest {
                id: 1,
                credit: account(1, 700),
                debit: account(2, 700),
                amount: 10,
            },
            TransferRequest {
                id: 2,
                credit: account(3, 800),
                debit: account(4, 800),
                amount: 10,
            },
        ];

        let result = bundle_transfers(&requests, 0);

        assert!(result.is_err());
    }

    #[test]
    fn rejects_mismatched_credit_debit_ledger_within_a_request() {
        let requests = vec![TransferRequest {
            id: 1,
            credit: account(1, 700),
            debit: account(2, 800),
            amount: 10,
        }];

        let result = bundle_transfers(&requests, 0);

        assert!(result.is_err());
    }

    #[test]
    fn builds_bundle_sharing_a_single_bundle_id() {
        let requests = vec![
            TransferRequest {
                id: 1,
                credit: account(1, 700),
                debit: account(2, 700),
                amount: 10,
            },
            TransferRequest {
                id: 2,
                credit: account(2, 700),
                debit: account(3, 700),
                amount: 20,
            },
        ];

        let bundle = bundle_transfers(&requests, 42).unwrap();

        assert_eq!(bundle.transactions.len(), 2);
        assert_eq!(bundle.transactions[0].bundle, bundle.transactions[1].bundle);
        assert_eq!(bundle.transactions[0].valid_from, 42);
        assert_eq!(bundle.transactions[1].id, 2);
        assert_eq!(bundle.transactions[1].credit, 2);
        assert_eq!(bundle.transactions[1].debit, 3);
        assert_eq!(bundle.transactions[1].amount, 20);
    }
}
