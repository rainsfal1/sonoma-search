-- Enable UUID extension
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Create webpages table with page_rank
CREATE TABLE IF NOT EXISTS webpages (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    url TEXT UNIQUE NOT NULL,
    domain TEXT NOT NULL,
    title TEXT,
    content_summary TEXT,
    fetch_timestamp TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    last_updated_timestamp TIMESTAMPTZ,
    status INTEGER,
    content_hash TEXT,
    metadata JSONB,
    meta_title TEXT,
    meta_description TEXT,
    meta_keywords TEXT,
    processed BOOLEAN DEFAULT FALSE,
    ranked BOOLEAN DEFAULT FALSE,
    last_ranked_at TIMESTAMPTZ,
    page_rank DOUBLE PRECISION DEFAULT 0.0
);

-- Add index for ranked status (moved up, before other indices)
CREATE INDEX IF NOT EXISTS idx_webpages_ranked ON webpages(ranked) WHERE ranked = FALSE;

-- Fix the text search index
CREATE INDEX IF NOT EXISTS idx_webpage_weighted_search ON webpages USING gin(
    (
        setweight(to_tsvector('english', coalesce(title, '')), 'A') ||
        setweight(to_tsvector('english', coalesce(content_summary, '')), 'B') ||
        setweight(to_tsvector('english', coalesce(meta_title, '')), 'C') ||
        setweight(to_tsvector('english', coalesce(meta_description, '')), 'D')
    )
);

-- Other indices
CREATE INDEX IF NOT EXISTS idx_webpages_url ON webpages(url);
CREATE INDEX IF NOT EXISTS idx_webpages_domain ON webpages(domain);
CREATE INDEX IF NOT EXISTS idx_webpages_processed ON webpages(processed) WHERE processed = FALSE;
CREATE INDEX IF NOT EXISTS idx_webpages_metadata ON webpages USING gin(metadata);
CREATE INDEX IF NOT EXISTS idx_webpages_page_rank ON webpages(page_rank);

-- Links table
CREATE TABLE IF NOT EXISTS links (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    source_webpage_id UUID REFERENCES webpages(id) ON DELETE CASCADE,
    target_url TEXT NOT NULL,
    anchor_text TEXT
);

CREATE INDEX IF NOT EXISTS idx_links_source_webpage_id ON links(source_webpage_id);
CREATE INDEX IF NOT EXISTS idx_links_target_url ON links(target_url);
CREATE INDEX IF NOT EXISTS idx_links_source_target ON links (source_webpage_id, target_url) 
WHERE source_webpage_id IS NOT NULL AND target_url LIKE 'http%';