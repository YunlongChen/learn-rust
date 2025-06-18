CREATE TABLE accounts
(
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    username        TEXT     NOT NULL UNIQUE,
    email           TEXT     NOT NULL,
    credential_type TEXT     NOT NULL,
    credential_data TEXT     NOT NULL,
    salt            TEXT     NOT NULL,
    created_at      DATETIME NOT NULL
);

CREATE TABLE domains
(
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    account_id  INTEGER NOT NULL,
    domain_name TEXT    NOT NULL,
    expire_ad   DATE,
    create_at   DATETIME,
    FOREIGN KEY (account_id) REFERENCES accounts (id) ON DELETE CASCADE
);

CREATE INDEX idx_domains_account ON domains (account_id);

CREATE TABLE domain_records
(
    id        INTEGER PRIMARY KEY AUTOINCREMENT,
    domain_id integer,
    type      TEXT NOT NULL,
    value     text not null,
    FOREIGN KEY (domain_id) REFERENCES domains (id) ON DELETE CASCADE
);

CREATE INDEX idx_domains_record ON domain_records (domain_id);