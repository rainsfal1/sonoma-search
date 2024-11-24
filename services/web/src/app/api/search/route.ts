import { NextResponse } from 'next/server'
import { search, SearchError, checkElasticsearchHealth } from '@/lib/elasticsearch'

export async function GET(request: Request) {
  const { searchParams } = new URL(request.url)
  const query = searchParams.get('q')
  const page = parseInt(searchParams.get('page') || '1')
  const size = parseInt(searchParams.get('size') || '10')

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

    const searchResults = await search(query, page, size)
    
    if (searchResults.total === 0) {
      return NextResponse.json({
        results: [],
        total: 0,
        took: 0,
        message: 'No results found. Our crawler may still be indexing content.'
      })
    }
    
    return NextResponse.json(searchResults)
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