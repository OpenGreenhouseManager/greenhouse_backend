-- This file should undo anything in `up.sql`
-- remove username unique constrain in users table
ALTER TABLE users
DROP CONSTRAINT unique_username;
