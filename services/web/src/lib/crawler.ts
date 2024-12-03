import axios from 'axios';

const CRAWLER_SERVICE_URL = process.env.NEXT_PUBLIC_CRAWLER_SERVICE_URL || 'http://crawler:8000';

export interface CrawlRequest {
  query: string;
  maxDepth?: number;
  maxPages?: number;
  priority?: boolean;
  force_crawl?: boolean;
}

export interface CrawlResponse {
  jobId: string;
  status: 'queued' | 'started' | 'error';
  message?: string;
}

export class CrawlerError extends Error {
  constructor(message: string, public readonly cause?: unknown) {
    super(message);
    this.name = 'CrawlerError';
  }
}

export async function triggerCrawl(
  query: string,
  options: Partial<CrawlRequest> = {}
): Promise<CrawlResponse> {
  try {
    const response = await axios.post<CrawlResponse>(`${CRAWLER_SERVICE_URL}/crawl`, {
      query,
      max_depth: options.maxDepth || 2,
      max_pages: options.maxPages || 100,
      priority: options.priority ?? true,
      force_crawl: options.force_crawl ?? false,
    });

    return response.data;
  } catch (error) {
    console.error('Failed to trigger crawl:', error);
    throw new CrawlerError(
      'Failed to start crawling process',
      error instanceof Error ? error : undefined
    );
  }
}

export async function checkCrawlStatus(jobId: string): Promise<{
  status: string;
  pagesProcessed: number;
  estimatedTimeRemaining?: number;
}> {
  try {
    const response = await axios.get(`${CRAWLER_SERVICE_URL}/status/${jobId}`);
    return response.data;
  } catch (error) {
    console.error('Failed to check crawl status:', error);
    throw new CrawlerError(
      'Failed to check crawl status',
      error instanceof Error ? error : undefined
    );
  }
}
