-- providers 表
CREATE TABLE providers
(
    id           UUID PRIMARY KEY,
    name         VARCHAR(255) NOT NULL,
    api_key      VARCHAR(255) NOT NULL,
    api_secret   VARCHAR(255) NOT NULL,
    extra_config JSONB,
    created_at   TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at   TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

-- domains 表
CREATE TABLE domains
(
    id          UUID PRIMARY KEY,
    name        VARCHAR(255) NOT NULL,
    provider_id UUID         NOT NULL REFERENCES providers (id) ON DELETE CASCADE,
    status      VARCHAR(50)  NOT NULL DEFAULT 'active',
    created_at  TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

-- dns_records 表
CREATE TABLE dns_records
(
    id          UUID PRIMARY KEY,
    domain_id   UUID         NOT NULL REFERENCES domains (id) ON DELETE CASCADE,
    record_type VARCHAR(10)  NOT NULL,
    name        VARCHAR(255) NOT NULL,
    value       TEXT         NOT NULL,
    ttl         INTEGER      NOT NULL,
    priority    INTEGER,
    created_at  TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);