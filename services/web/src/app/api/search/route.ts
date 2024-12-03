import { NextResponse } from 'next/server'
import { search, SearchError, checkElasticsearchHealth } from '@/lib/elasticsearch'
import { triggerCrawl } from '@/lib/crawler'

export async function GET(request: Request) {
  const { searchParams } = new URL(request.url)
  const query = searchParams.get('q')
  const page = parseInt(searchParams.get('page') || '1')
  const size = parseInt(searchParams.get('size') || '10')
  const waitForCrawl = searchParams.get('waitForCrawl') === 'true'

  if (!query) {
    return NextResponse.json(
      { error: 'Query parameter required' },
      { status: 400 }
    )
  }

  try {
    // First check if Elasticsearch is healthy
    const isHealthy = await checkElasticsearchHealth()
    if (!isHealthy) {
      return NextResponse.json(
        { error: 'Search service is currently unavailable' },
        { status: 503 }
      )
    }

    // Try searching first
    const searchResults = await search(query, page, size)
    
    if (searchResults.total === 0) {
      // Trigger crawling for queries with zero results
      try {
        const crawlResponse = await triggerCrawl(query, {
          maxDepth: 3,
          maxPages: 100,
          priority: true,
          force_crawl: true
        })

        // If waitForCrawl is true, wait a bit and try searching again
        if (waitForCrawl) {
          await new Promise(resolve => setTimeout(resolve, 5000)) // Wait 5 seconds
          const newResults = await search(query, page, size)
          
          if (newResults.total > 0) {
            return NextResponse.json({
              ...newResults,
              message: 'Found new results after crawling!'
            })
          }
        }

        return NextResponse.json({
          results: [],
          total: 0,
          took: 0,
          message: 'No results found yet. We\'ve started crawling for this query. Please try again in a few minutes.',
          crawlingStarted: true,
          jobId: crawlResponse.jobId
        })
      } catch (crawlError) {
        console.error('Failed to trigger crawl:', crawlError)
        return NextResponse.json({
          results: [],
          total: 0,
          took: 0,
          message: crawlError instanceof Error ? crawlError.message : 'Failed to start crawling. Please try again later.',
          crawlingStarted: false
        })
      }
    }

    return NextResponse.json({
      ...searchResults,
      page,
      totalPages: Math.ceil(searchResults.total / size)
    })
  } catch (error) {
    console.error('Search error:', error)
    
    if (error instanceof SearchError) {
      return NextResponse.json(
        { error: 'Search operation failed', details: error.message },
        { status: 503 }
      )
    }
    
    return NextResponse.json(
      { error: 'An unexpected error occurred' },
      { status: 500 }
    )
  }
}