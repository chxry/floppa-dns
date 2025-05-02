CREATE TABLE users (
    username TEXT PRIMARY KEY,
    pass_hash TEXT,
    created TIMESTAMP
);

CREATE TABLE domains (
    name TEXT PRIMARY KEY,
    ipv4 INET,
    ipv6 INET,
    owner TEXT REFERENCES users(username)
);

CREATE TABLE sessions (
    id UUID PRIMARY KEY,
    username TEXT REFERENCES users(username),
    created TIMESTAMP
);
