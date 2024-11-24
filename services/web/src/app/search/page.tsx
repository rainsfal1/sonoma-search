'use client';

import * as React from "react"
import { Moon, Sun, Search as SearchIcon, X, AlertCircle } from 'lucide-react'
import Link from "next/link"
import { useRouter, useSearchParams } from 'next/navigation'
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { useTheme } from "@/components/theme/ThemeProvider"
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert"
import { Skeleton } from "@/components/ui/skeleton"
import type { SearchResult } from "@/lib/elasticsearch"

function SearchPageContent() {
  const { theme, toggleTheme } = useTheme()
  const router = useRouter()
  const searchParams = useSearchParams()
  
  const [query, setQuery] = React.useState(searchParams.get('q') || '')
  const [results, setResults] = React.useState<SearchResult[]>([])
  const [total, setTotal] = React.useState(0)
  const [loading, setLoading] = React.useState(false)
  const [error, setError] = React.useState<string | null>(null)
  const [searchTime, setSearchTime] = React.useState(0)
  
  const handleSearch = React.useCallback(async (searchQuery: string) => {
    if (!searchQuery.trim()) {
      setResults([])
      setTotal(0)
      return
    }

    setLoading(true)
    setError(null)

    try {
      const response = await fetch(`/api/search?q=${encodeURIComponent(searchQuery)}`)
      if (!response.ok) {
        throw new Error('Search request failed')
      }
      const data = await response.json()
      setResults(data.results)
      setTotal(data.total)
      setSearchTime(data.took)
      router.push(`/search?q=${encodeURIComponent(searchQuery)}`)
    } catch (err) {
      setError('Search failed. Please try again later.')
      setResults([])
      setTotal(0)
    } finally {
      setLoading(false)
    }
  }, [router])

  // Initial search on mount if query param exists
  React.useEffect(() => {
    const initialQuery = searchParams.get('q')
    if (initialQuery) {
      handleSearch(initialQuery)
    }
  }, [searchParams, handleSearch])

  return (
    <div className="min-h-screen bg-background">
      <header className="sticky top-0 z-50 w-full border-b bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/60">
        <div className="container flex h-14 items-center">
          <div className="mr-4 hidden md:flex">
            <Link href="/" className="mr-6 flex items-center space-x-2">
              <span className="hidden font-bold sm:inline-block">Search Engine</span>
            </Link>
          </div>
          <div className="flex flex-1 items-center space-x-2">
            <form 
              className="flex-1 flex items-center space-x-2"
              onSubmit={(e) => {
                e.preventDefault()
                handleSearch(query)
              }}
            >
              <Input
                type="search"
                placeholder="Search..."
                className="flex-1"
                value={query}
                onChange={(e) => setQuery(e.target.value)}
              />
              <Button type="submit" disabled={loading}>
                {loading ? (
                  <div className="h-4 w-4 animate-spin rounded-full border-2 border-current border-t-transparent" />
                ) : (
                  <SearchIcon className="h-4 w-4" />
                )}
              </Button>
            </form>
            <Button
              variant="ghost"
              size="icon"
              onClick={() => toggleTheme()}
            >
              {theme === "light" ? (
                <Sun className="h-4 w-4" />
              ) : (
                <Moon className="h-4 w-4" />
              )}
            </Button>
          </div>
        </div>
      </header>

      <main className="container py-6">
        {error && (
          <Alert variant="destructive" className="mb-6">
            <AlertCircle className="h-4 w-4" />
            <AlertTitle>Error</AlertTitle>
            <AlertDescription>{error}</AlertDescription>
          </Alert>
        )}

        {loading ? (
          <div className="space-y-4">
            {Array.from({ length: 5 }).map((_, i) => (
              <div key={i} className="space-y-2">
                <Skeleton className="h-4 w-[250px]" />
                <Skeleton className="h-4 w-[450px]" />
                <Skeleton className="h-3 w-[200px]" />
              </div>
            ))}
          </div>
        ) : results.length > 0 ? (
          <>
            <p className="text-sm text-muted-foreground mb-4">
              About {total.toLocaleString()} results ({(searchTime / 1000).toFixed(2)} seconds)
            </p>
            <div className="space-y-6">
              {results.map((result, index) => (
                <article key={index} className="space-y-1">
                  <h2 className="text-lg font-semibold leading-snug">
                    <a 
                      href={result.url}
                      className="hover:underline"
                      target="_blank"
                      rel="noopener noreferrer"
                      dangerouslySetInnerHTML={{ __html: result.title }}
                    />
                  </h2>
                  <p className="text-sm text-muted-foreground">
                    {result.siteName} · {result.timestamp}
                  </p>
                  <p 
                    className="text-sm text-foreground/90"
                    dangerouslySetInnerHTML={{ __html: result.description }}
                  />
                </article>
              ))}
            </div>
          </>
        ) : query && !loading && (
          <p className="text-center text-muted-foreground py-8">
            No results found for "{query}"
          </p>
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
