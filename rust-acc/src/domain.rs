type Id = u128;
pub type GroupingId = u32;
type Amount = u128;
pub type Timestamp = u64;

pub type AccountId = Id;
pub type LedgerId = GroupingId;
pub struct Account {
    pub(crate) id: AccountId,

    pub(crate) credits: Amount,
    pub(crate) debits: Amount,

    pub(crate) ledger: LedgerId,
    pub(crate) code: GroupingId,

    pub(crate) created_at: Timestamp,
    pub(crate) deprecated_at: Option<Timestamp>,
}

impl Account {
    pub fn new(id: AccountId, ledger: LedgerId, code: GroupingId, created_at: Timestamp) -> Self {
        Self {
            id,
            credits: 0,
            debits: 0,
            ledger,
            code,
            created_at,
            deprecated_at: None,
        }
    }
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
