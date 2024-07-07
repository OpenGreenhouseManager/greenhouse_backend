CREATE TABLE users (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  username VARCHAR NOT NULL,
  password TEXT NOT NULL,
  salt TEXT NOT NULL,
  role TEXT NOT NULL
)