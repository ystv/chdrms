CREATE TABLE asset_types(
    id UUID PRIMARY KEY DEFAULT uuidv7(),
    name TEXT NOT NULL,
    manufacturer UUID REFERENCES manufacturers(id) NOT NULL,

    product_url TEXT,
    value NUMERIC(12, 2),

    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    created_by UUID REFERENCES users(id) NOT NULL
);
