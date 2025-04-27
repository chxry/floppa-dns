CREATE TABLE domains (
    name TEXT PRIMARY KEY,
    ip INET
);

CREATE TABLE users (
    username TEXT PRIMARY KEY,
    pass_hash TEXT,
    created TIMESTAMP
);

CREATE TABLE sessions (
    id UUID PRIMARY KEY,
    username TEXT REFERENCES users(username),
    created TIMESTAMP
);
