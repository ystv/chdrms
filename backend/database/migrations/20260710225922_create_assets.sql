CREATE TABLE asset_bundles(
    id UUID PRIMARY KEY DEFAULT uuidv7(),

    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    created_by UUID REFERENCES users(id) NOT NULL
);

CREATE TABLE assets(
    id UUID PRIMARY KEY DEFAULT uuidv7(),
    type UUID REFERENCES asset_types(id) NOT NULL,
    alias TEXT,
    tag TEXT NOT NULL UNIQUE,

    bundle UUID REFERENCES asset_bundles(id),

    home_location UUID REFERENCES locations(id) NOT NULL,
    location UUID REFERENCES locations(id) NOT NULL,

    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    created_by UUID REFERENCES users(id) NOT NULL
);
