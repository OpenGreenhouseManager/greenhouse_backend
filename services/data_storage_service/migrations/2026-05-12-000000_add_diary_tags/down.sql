DROP TRIGGER IF EXISTS trg_delete_orphan_diary_tag ON diary_entry_tag;
DROP FUNCTION IF EXISTS delete_orphan_diary_tag();
DROP TABLE diary_entry_tag;
DROP TABLE diary_tag;
