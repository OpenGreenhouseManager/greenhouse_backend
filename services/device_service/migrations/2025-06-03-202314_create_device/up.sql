-- Your SQL goes here
CREATE TABLE device (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  name VARCHAR NOT NULL,
  address VARCHAR NOT NULL,
  description VARCHAR NOT NULL,
  canScript boolean NOT NULL
);