CREATE TABLE group_permissions(
    group_id UUID REFERENCES groups(id),
    object TEXT NOT NULL,
    action TEXT NOT NULL,
    PRIMARY KEY(group_id, object, action)
);

ALTER TABLE users ADD COLUMN is_admin BOOLEAN NOT NULL DEFAULT false;
