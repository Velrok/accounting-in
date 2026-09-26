type Id = u128;
pub type GroupingId = u32;
pub type Amount = u128;
pub type Timestamp = u64;
pub type BundleId = u64;

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

pub type TransferId = Id;
pub struct Transfer {
    pub(crate) id: TransferId,

    // from -> to & how much
    pub(crate) credit: AccountId,
    pub(crate) debit: AccountId,
    pub(crate) amount: Amount,

    // parent_transfer means it belongs to the parent
    // all or nothing for commits
    pub(crate) bundle: BundleId,

    pub(crate) created_at: Timestamp,
    pub(crate) valid_from: Timestamp,
}
