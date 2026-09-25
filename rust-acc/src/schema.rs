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
