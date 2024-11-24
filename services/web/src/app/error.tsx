'use client'

import Image from "next/image"
import Link from "next/link"
import { Search, RefreshCw } from "lucide-react"
import { Button } from "@/components/ui/button"

export default function Error({
  error,
  reset,
}: {
  error: Error & { digest?: string }
  reset: () => void
}) {
  return (
    <div className="min-h-screen bg-background flex flex-col items-center justify-center p-4">
      <div className="text-center space-y-8 max-w-md">
        {/* Mascot */}
        <div className="relative w-64 h-64 mx-auto">
          <Image
            src="/images/mascot.jpg"
            alt="Sonoma Turtle"
            fill
            className="rounded-full shadow-lg object-cover"
            priority
          />
        </div>

        {/* Error Message */}
        <div className="space-y-3">
          <h1 className="text-4xl font-bold text-foreground">
            Oops!
          </h1>
          <p className="text-xl text-muted-foreground">
            Something went wrong
          </p>
          <p className="text-sm text-muted-foreground">
            {error.message || "Don't worry, we'll fix it!"}
          </p>
        </div>

        {/* Action Buttons */}
        <div className="flex items-center justify-center gap-4">
          <Button 
            onClick={reset}
            variant="outline"
            size="lg"
            className="flex items-center gap-2"
          >
            <RefreshCw className="h-4 w-4" />
            Try Again
          </Button>
          <Button 
            asChild
            size="lg"
          >
            <Link href="/" className="flex items-center gap-2">
              <Search className="h-4 w-4" />
              Back to Search
            </Link>
          </Button>
        </div>
      </div>
    </div>
  )
} 