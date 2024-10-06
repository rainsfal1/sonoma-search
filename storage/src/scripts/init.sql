-- Enable UUID extension
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Create webpages table
CREATE TABLE IF NOT EXISTS webpages (
                                        id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
                                        url TEXT UNIQUE NOT NULL,
                                        title TEXT,
                                        content TEXT,
                                        html_content TEXT,
                                        fetch_timestamp TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
                                        last_updated_timestamp TIMESTAMPTZ,
                                        status INTEGER,
                                        content_hash TEXT,
                                        metadata JSONB
);

-- Create indexes for webpages
CREATE INDEX IF NOT EXISTS idx_webpages_content_gin ON webpages
    USING gin(to_tsvector('english', coalesce(title, '') || ' ' || coalesce(content, '')));

CREATE INDEX IF NOT EXISTS idx_webpages_metadata ON webpages USING gin(metadata);
CREATE INDEX IF NOT EXISTS idx_webpages_url ON webpages(url);

-- Drop and recreate links table with UUID for id
DROP TABLE IF EXISTS links;

CREATE TABLE IF NOT EXISTS links (
                                     id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
                                     source_webpage_id UUID REFERENCES webpages(id) ON DELETE CASCADE,
                                     target_url TEXT NOT NULL,
                                     anchor_text TEXT,
                                     is_internal BOOLEAN
);

-- Create indexes for links
CREATE INDEX IF NOT EXISTS idx_links_source_webpage_id ON links(source_webpage_id);
CREATE INDEX IF NOT EXISTS idx_links_target_url ON links(target_url);