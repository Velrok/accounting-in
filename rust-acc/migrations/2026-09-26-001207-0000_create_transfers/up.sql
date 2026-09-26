CREATE TABLE transfers (
    id BLOB NOT NULL PRIMARY KEY,
    credit BLOB NOT NULL REFERENCES accounts (id),
    debit BLOB NOT NULL REFERENCES accounts (id),
    amount BLOB NOT NULL,
    bundle BIGINT NOT NULL,
    created_at BIGINT NOT NULL,
    valid_from BIGINT NOT NULL,
    CHECK (credit != debit),
    CHECK (amount > X'00000000000000000000000000000000')
);

CREATE UNIQUE INDEX transfers_id_unique ON transfers (id);
