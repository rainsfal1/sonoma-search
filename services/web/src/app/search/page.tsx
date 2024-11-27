'use client';

import * as React from "react"
import { Moon, Sun, Search as SearchIcon, X, AlertCircle, ChevronLeft, ChevronRight } from 'lucide-react'
import Link from "next/link"
import { useRouter, useSearchParams } from 'next/navigation'
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert"
import { Skeleton } from "@/components/ui/skeleton"
import { Pagination } from "@/components/ui/pagination"
import { CrawlerProgress } from "@/components/ui/crawler-progress"
import type { SearchResult } from "@/lib/elasticsearch"
import Image from 'next/image';

function SearchPageContent() {
  const router = useRouter()
  const searchParams = useSearchParams()
  
  const [query, setQuery] = React.useState(searchParams.get('q') || '')
  const [results, setResults] = React.useState<SearchResult[]>([])
  const [total, setTotal] = React.useState(0)
  const [loading, setLoading] = React.useState(false)
  const [error, setError] = React.useState<string | null>(null)
  const [searchTime, setSearchTime] = React.useState(0)
  const [currentPage, setCurrentPage] = React.useState(parseInt(searchParams.get('page') || '1'))
  const [totalPages, setTotalPages] = React.useState(0)
  const [crawlingStarted, setCrawlingStarted] = React.useState(false)
  const [message, setMessage] = React.useState<string | null>(null)
  const [isDark, setIsDark] = React.useState(false)
  
  React.useEffect(() => {
    // Check if dark mode is enabled
    const isDarkMode = document.documentElement.classList.contains('dark')
    setIsDark(isDarkMode)
  }, [])

  const toggleTheme = () => {
    const root = document.documentElement
    const newIsDark = !isDark
    if (newIsDark) {
      root.classList.add('dark')
    } else {
      root.classList.remove('dark')
    }
    setIsDark(newIsDark)
  }

  const handleSearch = React.useCallback(async (searchQuery: string, page: number = 1) => {
    if (!searchQuery.trim()) {
      setResults([])
      setTotal(0)
      return
    }

    setLoading(true)
    setError(null)
    setMessage(null)
    setCrawlingStarted(false)

    try {
      const response = await fetch(`/api/search?q=${encodeURIComponent(searchQuery)}&page=${page}`)
      if (!response.ok) {
        throw new Error('Search request failed')
      }
      const data = await response.json()
      
      if (data.error) {
        throw new Error(data.error)
      }

      setResults(data.results || [])
      setTotal(data.total || 0)
      setSearchTime(data.took / 1000) // Convert to seconds
      setTotalPages(data.totalPages || Math.ceil((data.total || 0) / 10))
      
      if (data.message) {
        setMessage(data.message)
      }
      
      if (data.crawlingStarted) {
        setCrawlingStarted(true)
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'An error occurred')
    } finally {
      setLoading(false)
    }
  }, [])

  React.useEffect(() => {
    const q = searchParams.get('q')
    const page = parseInt(searchParams.get('page') || '1')
    if (q) {
      setQuery(q)
      handleSearch(q, page)
    }
  }, [searchParams, handleSearch])

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault()
    if (query.trim()) {
      router.push(`/search?q=${encodeURIComponent(query)}`)
    }
  }

  const handlePageChange = (page: number) => {
    setCurrentPage(page)
    router.push(`/search?q=${encodeURIComponent(query)}&page=${page}`)
  }

  return (
    <div className="min-h-screen">
      <header className="sticky top-0 z-50 w-full border-b bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/60">
        <div className="container flex h-14 items-center gap-4">
          <Link href="/" className="flex items-center gap-2">
            <div className="relative w-8 h-8">
              <Image
                src="/images/mascot.jpg"
                alt="Sonoma Search Mascot"
                className="object-contain"
                width={32}
                height={32}
                priority
              />
            </div>
            <span className="font-bold text-primary">Sonoma</span>
          </Link>

          <form onSubmit={handleSubmit} className="flex-1 max-w-xl">
            <div className="relative">
              <Input
                type="search"
                placeholder="Search..."
                value={query}
                onChange={(e) => setQuery(e.target.value)}
                className="w-full pl-10 pr-4"
              />
              <SearchIcon className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
            </div>
          </form>

          <Button
            variant="ghost"
            size="icon"
            onClick={toggleTheme}
            className="ml-auto"
          >
            {!isDark ? <Sun className="h-5 w-5" /> : <Moon className="h-5 w-5" />}
            <span className="sr-only">Toggle theme</span>
          </Button>
        </div>
      </header>
      <main className="container py-6">
        {loading && !crawlingStarted && (
          <div className="flex justify-center py-8">
            <Skeleton className="h-12 w-12 rounded-full" />
          </div>
        )}

        {crawlingStarted && (
          <div className="flex flex-col items-center space-y-4 p-8 text-center">
            <div className="relative w-32 h-8">
              <div className="absolute inset-0">
                <div className="h-full bg-gradient-to-r from-background via-primary/20 to-background animate-shimmer" />
              </div>
              <Image
                src="/images/mascot.jpg"
                alt="Sonoma Search Mascot"
                className="object-contain absolute inset-0"
                width={32}
                height={32}
                priority
              />
            </div>
            <h3 className="text-lg font-semibold">Crawling the web for "{query}"...</h3>
            <div className="w-full max-w-xs">
              <div className="h-2 w-full rounded-full bg-secondary overflow-hidden">
                <div className="h-full bg-primary animate-progress" />
              </div>
            </div>
          </div>
        )}
        
        {error && (
          <Alert variant="destructive">
            <AlertCircle className="h-4 w-4" />
            <AlertTitle>Error</AlertTitle>
            <AlertDescription>{error}</AlertDescription>
          </Alert>
        )}
        
        {message && (
          <Alert>
            <AlertTitle>Notice</AlertTitle>
            <AlertDescription>{message}</AlertDescription>
          </Alert>
        )}
        
        {!loading && !crawlingStarted && results.length > 0 && (
          <>
            <div className="mb-4 text-sm text-muted-foreground">
              About {total.toLocaleString()} results ({searchTime.toFixed(2)} seconds)
            </div>
            <div className="space-y-6">
              {results.map((result, index) => (
                <div key={index} className="space-y-2">
                  <h2 className="text-xl font-semibold">
                    <a
                      href={result.url}
                      target="_blank"
                      rel="noopener noreferrer"
                      className="hover:underline"
                    >
                      {result.title || result.url}
                    </a>
                  </h2>
                  <div className="text-sm text-muted-foreground">{result.url}</div>
                  <p className="text-sm">{result.content_summary}</p>
                </div>
              ))}
            </div>
            {totalPages > 1 && (
              <div className="mt-8">
                <Pagination
                  currentPage={currentPage}
                  totalPages={totalPages}
                  onPageChange={handlePageChange}
                />
              </div>
            )}
          </>
        )}
        
        {!loading && !crawlingStarted && results.length === 0 && query && (
          <Alert>
            <AlertTitle>No Results Found</AlertTitle>
            <AlertDescription>
              Sorry, we couldn't find any results for your search query.
            </AlertDescription>
          </Alert>
        )}
      </main>
    </div>
  )
}

export default function SearchPage() {
  return (
    <React.Suspense fallback={<div>Loading...</div>}>
      <SearchPageContent />
    </React.Suspense>
  )
}
