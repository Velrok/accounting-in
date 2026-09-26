CREATE TABLE accounts (
    id BLOB NOT NULL PRIMARY KEY,
    credits BLOB NOT NULL,
    debits BLOB NOT NULL,
    ledger INTEGER NOT NULL,
    code INTEGER NOT NULL,
    created_at BIGINT NOT NULL,
    deprecated_at BIGINT
);
