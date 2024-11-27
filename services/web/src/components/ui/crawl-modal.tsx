'use client';

import * as React from "react";
import { X } from "lucide-react";
import Image from "next/image";
import { Button } from "./button";

interface CrawlModalProps {
  query: string;
  onClose: () => void;
  onStartCrawl: () => void;
  isLoading?: boolean;
  progress?: {
    pagesCrawled: number;
    status: string;
  };
}

export function CrawlModal({ query, onClose, onStartCrawl, isLoading, progress }: CrawlModalProps) {
  return (
    <div className="fixed inset-0 z-50 bg-background/80 backdrop-blur-sm">
      <div className="fixed left-[50%] top-[50%] z-50 grid w-full max-w-lg translate-x-[-50%] translate-y-[-50%] gap-4 border bg-background p-6 shadow-lg duration-200 sm:rounded-lg">
        <div className="flex flex-col space-y-1.5 text-center relative">
          <button 
            onClick={onClose}
            className="absolute right-0 top-0 p-2 hover:bg-muted rounded-sm transition-colors"
          >
            <X className="h-4 w-4" />
          </button>
          
          {!isLoading ? (
            <>
              <div className="mx-auto relative w-24 h-24 mb-4">
                <Image
                  src="/images/mascot.jpg"
                  alt="Sonoma Search Mascot"
                  className="object-contain"
                  width={96}
                  height={96}
                  priority
                />
              </div>
              <h2 className="text-lg font-semibold">No Results Found</h2>
              <p className="text-sm text-muted-foreground">
                Would you like me to crawl the web for "{query}"?
              </p>
              <div className="flex justify-center gap-2 mt-6">
                <Button variant="outline" onClick={onClose}>Cancel</Button>
                <Button onClick={onStartCrawl}>Start Crawling</Button>
              </div>
            </>
          ) : (
            <>
              <div className="flex flex-col items-center space-y-4">
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
                {progress && (
                  <p className="text-sm text-muted-foreground">
                    Found {progress.pagesCrawled} pages so far
                  </p>
                )}
              </div>
            </>
          )}
        </div>
      </div>
    </div>
  );
}
