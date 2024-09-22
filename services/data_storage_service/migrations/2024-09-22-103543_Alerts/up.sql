-- Your SQL goes here
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
    name VARCHAR(255) NOT NULL,
    value TEXT NOT NULL,
    note TEXT,
    start_at TIMESTAMP WITH TIME ZONE NOT NULL,
    end_at TIMESTAMP WITH TIME ZONE NOT NULL,
    data_source_id UUID REFERENCES data_source(id) NOT NULL
);