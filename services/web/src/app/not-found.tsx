import Image from "next/image"
import Link from "next/link"
import { Search, X } from "lucide-react"
import { Button } from "@/components/ui/button"
export default function NotFound() {
  return (
    <div className="min-h-screen bg-background flex flex-col items-center justify-center p-4">
      <div className="text-center space-y-8 max-w-md">
        {/* Mascot */}
        <div className="relative w-64 h-64 mx-auto">
          <Image
            src="/images/mascot.jpg"
            alt="Sonoma Turtle"
            fill
            className="object-contain"
            priority
          />
        </div>

        {/* Error Message */}
        <div className="space-y-3">
          <h1 className="text-4xl font-bold text-foreground">
            Oops!
          </h1>
          <p className="text-xl text-muted-foreground">
            Looks like this page took a swim in the wrong direction
          </p>
        </div>

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
  )
}