CREATE TABLE user_identities(
    user_id UUID REFERENCES users(id),
    provider TEXT NOT NULL,
    provider_id TEXT NOT NULL,
    PRIMARY KEY (user_id, provider),
    UNIQUE (provider, provider_id)
);
