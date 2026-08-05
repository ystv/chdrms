CREATE TYPE rgb AS (
    r SMALLINT,
    g SMALLINT,
    b SMALLINT
);

CREATE TABLE labels(
    id UUID PRIMARY KEY DEFAULT uuidv7(),
    name TEXT NOT NULL,
    description TEXT,
    colour rgb,

    blocking BOOLEAN NOT NULL DEFAULT false,

    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    created_by UUID REFERENCES users(id) NOT NULL
);

CREATE TABLE asset_labels(
    asset UUID REFERENCES assets(id) NOT NULL,
    label UUID REFERENCES labels(id) NOT NULL,

    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    created_by UUID REFERENCES users(id) NOT NULL,

    PRIMARY KEY (asset, label)
);
