CREATE TYPE severity AS ENUM ('fatal', 'error', 'warning', 'info');

CREATE TABLE alert (
    id UUID PRIMARY KEY,
    severity severity NOT NULL,
    identifier TEXT NOT NULL,
    value TEXT NOT NULL,
    note TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
    datasource_id UUID NOT NULL
);

-- Indexes for fast search
CREATE INDEX idx_identifier ON alert (identifier);
CREATE INDEX idx_data_source_id ON alert (datasource_id);