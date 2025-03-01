-- Your SQL goes here
CREATE TABLE diary_entry_alert (
    diary_entry_id UUID NOT NULL REFERENCES diary_entry(id) ON DELETE CASCADE,
    alert_id UUID NOT NULL REFERENCES alert(id) ON DELETE CASCADE,
    PRIMARY KEY (diary_entry_id, alert_id)
);

-- Create indexes for faster lookups
CREATE INDEX idx_diary_entry_alert_diary_entry_id ON diary_entry_alert (diary_entry_id);
CREATE INDEX idx_diary_entry_alert_alert_id ON diary_entry_alert (alert_id);
