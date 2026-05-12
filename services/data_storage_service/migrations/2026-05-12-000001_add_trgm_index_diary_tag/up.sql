CREATE EXTENSION IF NOT EXISTS pg_trgm;
CREATE INDEX idx_diary_tag_name_trgm ON diary_tag USING GIN (name gin_trgm_ops);
