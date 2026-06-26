CREATE TABLE manufacturers(
    id UUID PRIMARY KEY DEFAULT uuidv7(),
    name TEXT NOT NULL,
    description TEXT,

    website TEXT,
    email TEXT,
    phone TEXT,

    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    created_by UUID REFERENCES users(id) NOT NULL
);
