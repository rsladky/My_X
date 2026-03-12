ALTER TABLE users ADD COLUMN username TEXT NOT NULL DEFAULT '';
ALTER TABLE users ALTER COLUMN username DROP DEFAULT;
CREATE UNIQUE INDEX idx_users_username ON users(username);
