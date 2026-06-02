CREATE TABLE accounts
(
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    username        TEXT     NOT NULL UNIQUE,
    email           TEXT     NOT NULL,
    provider_type   TEXT     NOT NULL,
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
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    account_id  INTEGER,
    domain_id   INTEGER,
    record_name TEXT,
    record_type TEXT    NOT NULL,
    value       TEXT    NOT NULL,
    ttl         INTEGER NOT NULL,
    create_at   DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (domain_id) REFERENCES domains (id) ON DELETE CASCADE
);

CREATE INDEX idx_domains_record ON domain_records (domain_id);

--- 凭据信息
INSERT INTO accounts (id, username, email, provider_type, credential_type, credential_data, salt, created_at)
VALUES (2, 'aliyun', 'example@qq.com', 'Aliyun', 'ApiKey',
        '{"api_key":"exampleKey","api_secret":"exampleSecret"}', '',
        '2025-06-18 15:12:55.322183400 UTC');

-- 域名信息
INSERT INTO domains (id, account_id, domain_name, expire_ad, create_at)
VALUES (1, 2, 'example.cn', '2025-06-18 00:00:00.000', '2025-06-18 23:25:33.000');
INSERT INTO domains (id, account_id, domain_name, expire_ad, create_at)
VALUES (2, 2, 'example.xyz', '2025-06-18 00:00:00.000', '2025-06-18 23:25:48.000');

-- 域名记录
INSERT INTO domain_records (id, account_id, domain_id, record_name, record_type, value, ttl, create_at)
VALUES (1, 2, 1, 'www', 'A', '192.168.0.0', 0, current_timestamp);
INSERT INTO domain_records (id, account_id, domain_id, record_name, record_type, value, ttl, create_at)
VALUES (2, 2, 1, '@', '@', '192.168.0.0', 0, current_timestamp);


select id,id, account_id, domain_id, record_name, record_type, value, ttl, create_at from domain_records;