// @generated automatically by Diesel CLI.

diesel::table! {
    accounts (id) {
        id -> Binary,
        credits -> Binary,
        debits -> Binary,
        ledger -> Integer,
        code -> Integer,
        created_at -> BigInt,
        deprecated_at -> Nullable<BigInt>,
    }
}

diesel::table! {
    transfers (id) {
        id -> Binary,
        credit -> Binary,
        debit -> Binary,
        amount -> Binary,
        parent_transfer -> Nullable<Binary>,
        created_at -> BigInt,
        valid_from -> BigInt,
    }
}

diesel::allow_tables_to_appear_in_same_query!(accounts, transfers,);
