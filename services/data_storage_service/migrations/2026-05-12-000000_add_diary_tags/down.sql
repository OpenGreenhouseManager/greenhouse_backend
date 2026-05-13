DROP TRIGGER IF EXISTS trg_delete_orphan_diary_tag ON diary_entry_tag;
DROP FUNCTION IF EXISTS delete_orphan_diary_tag();
DROP INDEX IF EXISTS idx_diary_entry_tag_diary_tag_id;
DROP TABLE diary_entry_tag;
DROP TABLE diary_tag;
