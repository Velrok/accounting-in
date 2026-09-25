type Id = u128;
type GroupingId = u32;
type Amount = u128;
type Timestamp = u64;

type AccountId = Id;
type LedgerId = GroupingId;
pub struct Account {
    id: AccountId,
    ledger: LedgerId,
    code: GroupingId,
}

type TransferId = Id;
pub struct Transfer {
    id: TransferId,

    // from -> to & how much
    credit: AccountId,
    debit: AccountId,
    amount: Amount,

    // parent_transfer means it belongs to the parent
    // all or nothing for commits
    parent_transfer: Option<TransferId>,

    created_at: Timestamp,
    valid_from: Timestamp,
}
