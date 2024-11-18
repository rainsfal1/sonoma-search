-- Enable UUID extension
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Create webpages table
CREATE TABLE IF NOT EXISTS webpages (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    url TEXT UNIQUE NOT NULL,
    domain TEXT NOT NULL,
    title TEXT,
    content_summary TEXT,
    fetch_timestamp TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    last_updated_timestamp TIMESTAMPTZ,
    status INTEGER,
    content_hash TEXT,
    metadata JSONB,
    meta_title TEXT,
    meta_description TEXT,
    meta_keywords TEXT,
    processed bool default false,
    page_rank double precision default 0.0
);

-- Create indexes for webpages
CREATE INDEX IF NOT EXISTS idx_webpages_content_gin ON webpages
    USING gin(to_tsvector('english', coalesce(title, '') || ' ' || coalesce(content_summary, '')));
CREATE INDEX IF NOT EXISTS idx_webpages_url ON webpages(url);
CREATE INDEX IF NOT EXISTS idx_webpages_domain ON webpages(domain);
CREATE INDEX IF NOT EXISTS idx_webpages_metadata ON webpages USING gin(metadata);
CREATE INDEX IF NOT EXISTS idx_webpages_meta_title ON webpages USING gin(to_tsvector('english', coalesce(meta_title, '')));
CREATE INDEX IF NOT EXISTS idx_webpages_meta_description ON webpages USING gin(to_tsvector('english', coalesce(meta_description, '')));
CREATE INDEX IF NOT EXISTS idx_webpages_meta_keywords ON webpages USING gin(to_tsvector('english', coalesce(meta_keywords, '')));

-- Create links table
CREATE TABLE IF NOT EXISTS links (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    source_webpage_id UUID REFERENCES webpages(id) ON DELETE CASCADE,
    target_url TEXT NOT NULL,
    anchor_text TEXT
);

-- Create indexes for links
CREATE INDEX IF NOT EXISTS idx_links_source_webpage_id ON links(source_webpage_id);
CREATE INDEX IF NOT EXISTS idx_links_target_url ON links(target_url);


