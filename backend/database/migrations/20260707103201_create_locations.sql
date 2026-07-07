CREATE TABLE locations(
    id UUID PRIMARY KEY DEFAULT uuidv7(),
    name TEXT NOT NULL,
    description TEXT,

    coordinates POINT,

    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    created_by UUID REFERENCES users(id) NOT NULL
);
