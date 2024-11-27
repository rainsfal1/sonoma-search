import * as React from "react"
import { ChevronLeft, ChevronRight, MoreHorizontal } from "lucide-react"
import { cn } from "@/lib/utils"
import { Button } from "@/components/ui/button"

interface PaginationProps {
  currentPage: number
  totalPages: number
  onPageChange: (page: number) => void
  maxVisible?: number
}

export function Pagination({
  currentPage,
  totalPages,
  onPageChange,
  maxVisible = 5,
}: PaginationProps) {
  const getPageNumbers = () => {
    const pages: (number | string)[] = []
    const halfVisible = Math.floor(maxVisible / 2)
    
    // Always show first page
    pages.push(1)
    
    if (currentPage > halfVisible + 2) {
      pages.push('...')
    }
    
    // Calculate range around current page
    let rangeStart = Math.max(2, currentPage - halfVisible)
    let rangeEnd = Math.min(totalPages - 1, currentPage + halfVisible)
    
    // Adjust range if at the start or end
    if (currentPage <= halfVisible + 2) {
      rangeEnd = Math.min(totalPages - 1, maxVisible)
    }
    if (currentPage >= totalPages - halfVisible - 1) {
      rangeStart = Math.max(2, totalPages - maxVisible + 1)
    }
    
    // Add range pages
    for (let i = rangeStart; i <= rangeEnd; i++) {
      pages.push(i)
    }
    
    // Add ellipsis and last page if needed
    if (currentPage < totalPages - halfVisible - 1) {
      if (rangeEnd < totalPages - 1) pages.push('...')
    }
    if (totalPages > 1) pages.push(totalPages)
    
    return pages
  }

  const pages = getPageNumbers()

  return (
    <nav role="navigation" aria-label="Pagination Navigation" className="flex items-center space-x-1">
      <Button
        variant="outline"
        size="icon"
        onClick={() => onPageChange(currentPage - 1)}
        disabled={currentPage === 1}
        aria-label="Previous page"
      >
        <ChevronLeft className="h-4 w-4" />
      </Button>
      
      {pages.map((page, i) => {
        if (page === '...') {
          return (
            <Button
              key={`ellipsis-${i}`}
              variant="ghost"
              size="icon"
              disabled
              className="cursor-default"
            >
              <MoreHorizontal className="h-4 w-4" />
            </Button>
          )
        }
        
        const pageNum = page as number
        return (
          <Button
            key={pageNum}
            variant={currentPage === pageNum ? "default" : "outline"}
            onClick={() => onPageChange(pageNum)}
            aria-label={`Page ${pageNum}`}
            aria-current={currentPage === pageNum ? "page" : undefined}
            className={cn(
              "min-w-[2.25rem] px-3",
              currentPage === pageNum && "pointer-events-none"
            )}
          >
            {pageNum}
          </Button>
        )
      })}
      
      <Button
        variant="outline"
        size="icon"
        onClick={() => onPageChange(currentPage + 1)}
        disabled={currentPage === totalPages}
        aria-label="Next page"
      >
        <ChevronRight className="h-4 w-4" />
      </Button>
    </nav>
  )
}
