-- Create webpages table
CREATE TABLE IF NOT EXISTS webpages (
                                        id UUID PRIMARY KEY,
                                        url TEXT UNIQUE NOT NULL,
                                        title TEXT,
                                        content TEXT,
                                        html_content TEXT,
                                        fetch_timestamp TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
                                        last_updated_timestamp TIMESTAMP WITH TIME ZONE,
                                        status INTEGER,
                                        content_hash TEXT,
                                        metadata JSONB
);

-- Create index for full-text search on 'title' and 'content'
CREATE INDEX IF NOT EXISTS idx_webpages_content_gin ON webpages
    USING gin(to_tsvector('english', coalesce(title, '') || ' ' || coalesce(content, '')));

-- Create GIN index on 'metadata' to efficiently query JSONB
CREATE INDEX IF NOT EXISTS idx_webpages_metadata ON webpages USING gin(metadata);

-- Create index on 'url' for faster lookups
CREATE INDEX IF NOT EXISTS idx_webpages_url ON webpages(url);


-- Create links table
CREATE TABLE IF NOT EXISTS links (
                                     id BIGSERIAL PRIMARY KEY,  -- Use BIGSERIAL for large-scale data handling
                                     source_webpage_id UUID REFERENCES webpages(id) ON DELETE CASCADE,
                                     target_url TEXT NOT NULL,
                                     anchor_text TEXT,
                                     is_internal BOOLEAN
);

-- Create index for faster lookups by 'source_webpage_id'
CREATE INDEX IF NOT EXISTS idx_links_source_webpage_id ON links(source_webpage_id);

-- Create index for faster lookups by 'target_url'
CREATE INDEX IF NOT EXISTS idx_links_target_url ON links(target_url);
