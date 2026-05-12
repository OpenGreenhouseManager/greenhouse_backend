CREATE TABLE diary_tag (
    id UUID PRIMARY KEY,
    name TEXT NOT NULL UNIQUE
);

CREATE TABLE diary_entry_tag (
    diary_entry_id UUID NOT NULL REFERENCES diary_entry(id) ON DELETE CASCADE,
    diary_tag_id   UUID NOT NULL REFERENCES diary_tag(id)   ON DELETE CASCADE,
    PRIMARY KEY (diary_entry_id, diary_tag_id)
);

-- Automatically delete a tag when it is no longer referenced by any entry.
-- Fires after every delete on diary_entry_tag (including cascades from diary_entry).
CREATE OR REPLACE FUNCTION delete_orphan_diary_tag()
RETURNS TRIGGER AS $$
BEGIN
    DELETE FROM diary_tag
    WHERE id = OLD.diary_tag_id
      AND NOT EXISTS (
          SELECT 1 FROM diary_entry_tag WHERE diary_tag_id = OLD.diary_tag_id
      );
    RETURN OLD;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_delete_orphan_diary_tag
AFTER DELETE ON diary_entry_tag
FOR EACH ROW EXECUTE FUNCTION delete_orphan_diary_tag();
