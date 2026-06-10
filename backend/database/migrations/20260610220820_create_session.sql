CREATE TABLE user_sessions(
    token UUID PRIMARY KEY DEFAULT uuidv7(),
    user_id UUID references users(id)
);
