'use client';

import * as React from "react";
import { Loader2 } from "lucide-react";

interface CrawlerProgressProps {
  query: string;
  onComplete?: () => void;
}

export function CrawlerProgress({ query, onComplete }: CrawlerProgressProps) {
  const [progress, setProgress] = React.useState(0);
  const [pagesCrawled, setPagesCrawled] = React.useState(0);
  const [timeRemaining, setTimeRemaining] = React.useState<number | null>(null);

  React.useEffect(() => {
    let interval: NodeJS.Timeout;
    const checkProgress = async () => {
      try {
        const response = await fetch(`/api/crawl/status`);
        const data = await response.json();
        
        if (data.pages_crawled) {
          setPagesCrawled(data.pages_crawled);
          // Assuming we want to crawl at least 50 pages
          setProgress(Math.min((data.pages_crawled / 50) * 100, 100));
          setTimeRemaining(data.estimated_remaining);
        }

        if (data.status === "completed") {
          clearInterval(interval);
          onComplete?.();
        }
      } catch (error) {
        console.error("Error checking crawl status:", error);
      }
    };

    interval = setInterval(checkProgress, 1000);
    return () => clearInterval(interval);
  }, [onComplete]);

  return (
    <div className="flex flex-col items-center space-y-4 p-8 text-center">
      <div className="relative h-32 w-32">
        {/* Cute spider web animation */}
        <div className="absolute inset-0 animate-spin-slow">
          {[...Array(8)].map((_, i) => (
            <div
              key={i}
              className="absolute h-full w-0.5 bg-gradient-to-b from-transparent via-foreground/20 to-transparent"
              style={{
                transform: `rotate(${i * 45}deg)`,
              }}
            />
          ))}
        </div>
        {/* Bouncing spider */}
        <div className="absolute inset-0 flex items-center justify-center">
          <Loader2 className="h-8 w-8 animate-bounce text-foreground" />
        </div>
      </div>
      <h3 className="text-lg font-semibold">Crawling the web for "{query}"...</h3>
      <div className="w-full max-w-xs">
        <div className="h-2 w-full rounded-full bg-secondary">
          <div
            className="h-full rounded-full bg-primary transition-all duration-500"
            style={{ width: `${progress}%` }}
          />
        </div>
      </div>
      <p className="text-sm text-muted-foreground">
        Found {pagesCrawled} pages so far
        {timeRemaining && timeRemaining > 0 && (
          <> • About {Math.ceil(timeRemaining)} seconds remaining</>
        )}
      </p>
    </div>
  );
}
