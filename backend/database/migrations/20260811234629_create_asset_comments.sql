CREATE TABLE asset_comments(
    asset UUID REFERENCES assets(id) NOT NULL,
    comment UUID REFERENCES comments(id) NOT NULL
);
