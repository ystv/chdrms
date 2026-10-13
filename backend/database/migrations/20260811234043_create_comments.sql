CREATE TABLE comments(
    id UUID PRIMARY KEY DEFAULT uuidv7(),
    archived_at TIMESTAMPTZ,

    title TEXT NOT NULL,
    content TEXT NOT NULL,

    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    created_by UUID REFERENCES users(id) NOT NULL
);
