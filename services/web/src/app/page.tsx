'use client';

import * as React from "react"
import { Moon, Sun, Search, Info, CloudSun, X } from 'lucide-react'
import Image from "next/image"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from "@/components/ui/tooltip"
import { CrawlModal } from "@/components/ui/crawl-modal"
import { InfoModal } from "@/components/ui/info-modal"
import { FeelingLuckyModal } from "@/components/ui/feeling-lucky-modal"

function getCommandKey() {
  if (typeof navigator === 'undefined') return '⌘'
  const platform = navigator.platform.toLowerCase()
  const isMac = platform.includes('mac') || platform.includes('iphone') || platform.includes('ipad')
  return isMac ? '⌘' : 'Ctrl'
}

export default function Home() {
  const [isDark, setIsDark] = React.useState(false)
  const [weather, setWeather] = React.useState({ temp: "--", condition: "Loading..." })
  const [searchValue, setSearchValue] = React.useState('')
  const [showCrawlModal, setShowCrawlModal] = React.useState(false)
  const [showInfoModal, setShowInfoModal] = React.useState(false)
  const [showLuckyModal, setShowLuckyModal] = React.useState(false)
  const [isCrawling, setIsCrawling] = React.useState(false)
  const [crawlProgress, setCrawlProgress] = React.useState<{ pagesCrawled: number; status: string; } | null>(null)
  const [isClosing, setIsClosing] = React.useState(false);
  const inputRef = React.useRef<HTMLInputElement>(null)

  React.useEffect(() => {
    const down = (e: KeyboardEvent) => {
      if (e.key === 'k' && (e.metaKey || e.ctrlKey)) {
        e.preventDefault()
        inputRef.current?.focus()
      }
      if (e.key === 'Escape') {
        if (showInfoModal) closeInfoModal();
        if (showLuckyModal) closeLuckyModal();
      }
    }

    document.addEventListener('keydown', down)
    return () => document.removeEventListener('keydown', down)
  }, [showInfoModal, showLuckyModal])

  const toggleTheme = () => {
    setIsDark(!isDark)
    document.documentElement.classList.toggle("dark")
  }

  React.useEffect(() => {
    setTimeout(() => {
      setWeather({ temp: "72°F", condition: "Partly Cloudy" })
    }, 1000)
  }, [])

  const handleSearch = async (e: React.FormEvent) => {
    e.preventDefault()
    if (!searchValue.trim()) return

    try {
      const response = await fetch(`/api/search?q=${encodeURIComponent(searchValue.trim())}`)
      const data = await response.json()

      if (data.results && data.results.length > 0) {
        // Has results, redirect to search page
        window.location.href = `/search?q=${encodeURIComponent(searchValue.trim())}`
      } else {
        // No results, show crawl modal
        setShowCrawlModal(true)
      }
    } catch (error) {
      console.error('Search error:', error)
    }
  }

  const startCrawling = async () => {
    setIsCrawling(true)
    
    try {
      // Start crawling
      const response = await fetch('/api/crawl', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ query: searchValue.trim() })
      })
      const { job_id } = await response.json()

      // Poll for progress
      const interval = setInterval(async () => {
        const statusResponse = await fetch(`/api/crawl/status/${job_id}`)
        const status = await statusResponse.json()
        
        setCrawlProgress({
          pagesCrawled: status.pages_crawled,
          status: status.status
        })

        if (status.status === 'completed') {
          clearInterval(interval)
          window.location.href = `/search?q=${encodeURIComponent(searchValue.trim())}`
        }
      }, 1000)

      return () => clearInterval(interval)
    } catch (error) {
      console.error('Crawl error:', error)
      setIsCrawling(false)
    }
  }

  const closeInfoModal = () => {
    setIsClosing(true);
    setTimeout(() => {
      setShowInfoModal(false);
      setIsClosing(false);
    }, 300);
  };

  const closeLuckyModal = () => {
    setIsClosing(true);
    setTimeout(() => {
      setShowLuckyModal(false);
      setIsClosing(false);
    }, 300);
  };

  return (
    <div className={`min-h-screen w-full transition-colors ${isDark ? "dark" : ""}`}>
      <div className="flex min-h-screen flex-col items-center justify-center gap-8 bg-background px-4 transition-colors relative">
        <div className="absolute inset-0 overflow-hidden">
          <div className="absolute -inset-[10px] opacity-50">
            {[...Array(20)].map((_, i) => {
              const top = ((i * 17) % 100);
              const left = ((i * 23) % 100);
              const duration = 15 + (i % 10);
              const delay = -(i * 1.5);
              
              return (
                <div
                  key={i}
                  className="absolute h-1 w-1 rounded-full bg-gray-500/10 dark:bg-gray-400/10"
                  style={{
                    top: `${top}%`,
                    left: `${left}%`,
                    animation: `float ${duration}s linear infinite`,
                    animationDelay: `${delay}s`,
                  }}
                />
              );
            })}
          </div>
        </div>

        <header className="absolute left-4 top-4">
          <div className="group flex items-center gap-3 text-sm text-muted-foreground rounded-full bg-background/50 backdrop-blur-sm border border-transparent hover:border-border p-3 px-4 transition-all duration-300">
            <CloudSun className="h-5 w-5 transition-transform group-hover:scale-110 group-hover:text-primary" />
            <div className="flex flex-col gap-0.5">
              <span className="group-hover:text-foreground transition-colors">{weather.temp}</span>
              <span className="text-xs group-hover:text-foreground transition-colors">{weather.condition}</span>
            </div>
          </div>
        </header>

        <header className="absolute right-4 top-4 flex items-center gap-4">
          <Button 
            variant="ghost" 
            size="icon" 
            onClick={() => setShowInfoModal(true)}
          >
            <Info className="h-5 w-5" />
            <span className="sr-only">About Sonoma</span>
          </Button>
          <Button variant="ghost" size="icon" onClick={toggleTheme}>
            {isDark ? <Sun className="h-5 w-5" /> : <Moon className="h-5 w-5" />}
            <span className="sr-only">Toggle theme</span>
          </Button>
        </header>

        <div className="flex flex-col items-center">
          <div className="relative w-44 h-44 sm:w-56 sm:h-56 flex items-center justify-center transition-transform duration-300 hover:scale-105">
            <Image
              src="/images/mascot.jpg"
              alt="Sonoma Search Mascot"
              className="object-contain"
              width={200}
              height={200}
              priority
            />
          </div>
          <h1 className="text-4xl font-bold tracking-tighter text-primary -mt-6">
            Sonoma
          </h1>
        </div>

        <div className="w-full max-w-xl space-y-4">
          <form onSubmit={handleSearch}>
            <div className="relative">
              <Input
                ref={inputRef}
                name="q"
                value={searchValue}
                onChange={(e) => setSearchValue(e.target.value)}
                className="h-14 rounded-md bg-background pl-12 pr-12 shadow-lg transition-all duration-300 border-[#E4E4E7] dark:border-input focus:ring-2 focus:ring-primary focus:ring-offset-2 focus:ring-offset-background text-lg [&::-webkit-search-cancel-button]:hidden"
                placeholder="Search the web..."
                type="text"
              />
              <Search className="absolute left-4 top-1/2 h-6 w-6 -translate-y-1/2 text-muted-foreground animate-pulse-slow" />
              
              <div className="absolute right-4 top-1/2 -translate-y-1/2 flex items-center gap-1">
                {searchValue ? (
                  <button
                    type="button"
                    onClick={() => setSearchValue('')}
                    className="p-1.5 hover:bg-muted rounded-sm text-muted-foreground hover:text-foreground transition-colors"
                  >
                    <X className="h-4 w-4" />
                    <span className="sr-only">Clear search</span>
                  </button>
                ) : (
                  <kbd className="hidden md:inline-flex h-5 items-center gap-1 rounded border bg-muted px-1.5 text-[10px] font-medium text-muted-foreground">
                    <span className="translate-y-[1px]">{getCommandKey()}</span>K
                  </kbd>
                )}
              </div>
            </div>
            <div className="flex justify-center gap-4 mt-4">
              <Button 
                type="submit" 
                className="h-11 px-6 text-base relative group bg-[#F4F4F5] dark:bg-secondary hover:bg-[#E4E4E7] dark:hover:bg-secondary/80 text-[#18181B] dark:text-white transition-all duration-300 hover:scale-105 hover:shadow-md"
              >
                <span className="relative z-10 flex items-center gap-2">
                  <Search className="h-4 w-4" />
                  Sonoma Search
                </span>
              </Button>
              <Button 
                type="button" 
                onClick={() => setShowLuckyModal(true)}
                className="h-11 px-6 text-base relative group bg-[#F4F4F5] dark:bg-secondary hover:bg-[#E4E4E7] dark:hover:bg-secondary/80 text-[#18181B] dark:text-white transition-all duration-300 hover:scale-105 hover:shadow-md"
              >
                <span className="relative z-10 flex items-center gap-2">
                  <span role="img" aria-label="club" className="text-sm">♣️</span>
                  I&apos;m Feeling Lucky
                </span>
              </Button>
            </div>
          </form>
        </div>

        <footer className="absolute bottom-4 text-sm text-muted-foreground">
          &copy; 2024 Sonoma Ltd. All rights reserved.
        </footer>
      </div>
      {showCrawlModal && (
        <CrawlModal
          query={searchValue}
          onClose={() => {
            setShowCrawlModal(false)
            setIsCrawling(false)
            setCrawlProgress(null)
          }}
          onStartCrawl={startCrawling}
          isLoading={isCrawling}
          progress={crawlProgress}
        />
      )}

      <div 
        className={`fixed inset-0 z-50 transition-opacity duration-300 ${showInfoModal && !isClosing ? 'opacity-100' : 'opacity-0 pointer-events-none'}`}
      >
        {(showInfoModal || isClosing) && <InfoModal onClose={closeInfoModal} />}
      </div>

      <div 
        className={`fixed inset-0 z-50 transition-opacity duration-300 ${showLuckyModal && !isClosing ? 'opacity-100' : 'opacity-0 pointer-events-none'}`}
      >
        {(showLuckyModal || isClosing) && <FeelingLuckyModal onClose={closeLuckyModal} />}
      </div>
    </div>
  )
}