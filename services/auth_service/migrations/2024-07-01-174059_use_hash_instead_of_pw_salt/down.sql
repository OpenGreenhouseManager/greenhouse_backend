-- This file should undo anything in `up.sql`
ALTER TABLE users
DROP COLUMN hash;
ALTER TABLE users
ADD COLUMN password TEXT NOT NULL;  
ALTER TABLE users
ADD COLUMN salt TEXT NOT NULL;