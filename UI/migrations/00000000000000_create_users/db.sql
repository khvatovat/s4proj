-- Create users table
CREATE TABLE users (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    username TEXT NOT NULL UNIQUE,
    fingerprint BLOB
);

-- Create credentials table
CREATE TABLE credentials (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL,
    site TEXT NOT NULL,
    site_username TEXT NOT NULL,
    site_password TEXT NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users(id)
);
