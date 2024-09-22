CREATE TYPE severity AS ENUM ('fatal', 'error', 'warning', 'info');

CREATE TABLE data_source (
    id UUID PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    type VARCHAR(255) NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL
);

CREATE TABLE alert (
    id UUID PRIMARY KEY,
    severity severity NOT NULL,
    identifier UUID NOT NULL,
    value TEXT NOT NULL,
    note TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
    datasource_id UUID NOT NULL,
    FOREIGN KEY (id) REFERENCES data_source(id)
);

-- Indexes for fast search
CREATE INDEX idx_identifier ON alert (identifier);
CREATE INDEX idx_data_source_id ON alert (datasource_id);