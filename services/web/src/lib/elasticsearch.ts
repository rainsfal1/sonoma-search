import { Client } from '@elastic/elasticsearch'
import { formatDistanceToNow } from 'date-fns'

const client = new Client({
  node: process.env.ELASTICSEARCH_URL || 'http://localhost:9200',
  maxRetries: 3,
  requestTimeout: 10000,
  sniffOnStart: true
})

interface ElasticsearchSource {
  webpage_id: string;
  title: string;
  body: string;
  indexed_at: string;
  metadata: {
    site_name: string;
    language: string;
  };
  content_summary: string;
  keywords: string[];
  page_rank: number;
  meta_title?: string;
  meta_description?: string;
}

interface ElasticsearchHit {
  _source: ElasticsearchSource;
  _score: number;
  highlight?: {
    title?: string[];
    meta_description?: string[];
    body?: string[];
  };
}

export interface SearchResult {
  title: string;
  url: string;
  description: string;
  siteName: string;
  timestamp: string;
  page_rank: number;
  score: number;
  content_summary: string;
}

export interface SearchResponse {
  results: SearchResult[];
  total: number;
  took: number;
}

export class SearchError extends Error {
  constructor(message: string, public readonly cause?: unknown) {
    super(message);
    this.name = 'SearchError';
  }
}

export async function search(query: string, page = 1, size = 10): Promise<SearchResponse> {
  try {
    const startTime = Date.now();
    const response = await client.search({
      index: 'pages',
      body: {
        query: {
          bool: {
            must: {
              multi_match: {
                query,
                fields: [
                  'title^4',
                  'meta_title^3',
                  'meta_description^2.5',
                  'content_summary^2',
                  'body^1'
                ],
                type: 'best_fields',
                tie_breaker: 0.3,
                minimum_should_match: '80%'
              }
            },
            should: [
              { exists: { field: "page_rank", boost: 1.5 } },
              { range: { page_rank: { gt: 0, boost: 2.0 } } }
            ]
          }
        },
        size,
        from: (page - 1) * size,
        sort: [
          "_score",
          { page_rank: { order: "desc", missing: "_last" } }
        ],
        highlight: {
          pre_tags: ['<b>'],
          post_tags: ['</b>'],
          fields: {
            title: { 
              number_of_fragments: 0,
              type: 'unified'
            },
            meta_description: {
              number_of_fragments: 1,
              fragment_size: 200,
              type: 'unified'
            },
            body: {
              number_of_fragments: 1,
              fragment_size: 200,
              type: 'unified'
            }
          }
        }
      }
    });

    const hits = response.hits as unknown as { 
      hits: ElasticsearchHit[],
      total: { value: number }
    };

    return {
      results: hits.hits.map(hit => {
        // Safely parse the date with fallback
        let timestamp = 'Unknown time';
        try {
          if (hit._source.indexed_at) {
            const date = new Date(hit._source.indexed_at);
            if (!isNaN(date.getTime())) {
              timestamp = formatDistanceToNow(date, { addSuffix: true });
            }
          }
        } catch (e) {
          console.warn('Failed to parse date:', hit._source.indexed_at);
        }

        // Get the best description from available fields
        const description = hit.highlight?.meta_description?.[0] || 
                          hit.highlight?.body?.[0] || 
                          hit._source.meta_description ||
                          hit._source.content_summary ||
                          '';

        // Clean up description
        const cleanDescription = description
          .replace(/\[\s*edit\s*\]/gi, '')
          .replace(/Jump to (navigation|search)/gi, '')
          .trim();

        return {
          title: hit.highlight?.title?.[0] || hit._source.title || 'Untitled',
          url: hit._source.webpage_id || '',
          description: cleanDescription,
          siteName: hit._source.metadata?.site_name || 
            (hit._source.webpage_id.startsWith('http') ? new URL(hit._source.webpage_id).hostname : 'Unknown'),
          timestamp,
          page_rank: hit._source.page_rank || 0,
          score: hit._score,
          content_summary: hit._source.content_summary || ''
        };
      }),
      total: hits.total.value,
      took: Date.now() - startTime
    }
  } catch (error) {
    console.error('Elasticsearch search error:', error);
    throw new SearchError('Failed to perform search', error);
  }
}

// Health check function
export async function checkElasticsearchHealth(): Promise<boolean> {
  try {
    const response = await client.cluster.health();
    return ['green', 'yellow'].includes(response.status);
  } catch (error) {
    console.error('Elasticsearch health check failed:', error);
    return false;
  }
}