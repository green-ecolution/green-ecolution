import * as React from 'react'
import { ChevronLeft, ChevronRight, MoreHorizontal } from 'lucide-react'

import { cn } from '@/lib/utils'
import { Button, type ButtonProps } from '@/components/ui/button'

const Pagination = ({ className, ...props }: React.ComponentProps<'nav'>) => (
  <nav
    role="navigation"
    aria-label="pagination"
    data-slot="pagination"
    className={cn('mx-auto flex w-full justify-center', className)}
    {...props}
  />
)
Pagination.displayName = 'Pagination'

const PaginationContent = React.forwardRef<HTMLUListElement, React.ComponentProps<'ul'>>(
  ({ className, ...props }, ref) => (
    <ul ref={ref} data-slot="pagination-content" className={cn('flex flex-row items-center gap-1', className)} {...props} />
  ),
)
PaginationContent.displayName = 'PaginationContent'

const PaginationItem = React.forwardRef<HTMLLIElement, React.ComponentProps<'li'>>(
  ({ className, ...props }, ref) => <li ref={ref} data-slot="pagination-item" className={cn('', className)} {...props} />,
)
PaginationItem.displayName = 'PaginationItem'

type PaginationLinkProps = {
  isActive?: boolean
} & Pick<ButtonProps, 'size'> &
  React.ComponentProps<'a'>

const PaginationLink = ({ className, isActive, size = 'icon', ...props }: PaginationLinkProps) => (
  <a
    data-slot="pagination-link"
    aria-current={isActive ? 'page' : undefined}
    className={cn(
      'inline-flex h-10 w-10 items-center justify-center rounded-lg text-sm font-medium transition-colors',
      'hover:bg-dark-100 hover:text-dark-800',
      'focus-visible:outline-none focus-visible:ring-[3px] focus-visible:ring-green-dark/50',
      isActive && 'bg-green-dark text-white hover:bg-green-dark/90',
      className,
    )}
    {...props}
  />
)
PaginationLink.displayName = 'PaginationLink'

const PaginationPrevious = ({
  className,
  ...props
}: React.ComponentProps<typeof PaginationLink>) => (
  <PaginationLink data-slot="pagination-previous" aria-label="Zur vorherigen Seite" size="icon" className={className} {...props}>
    <ChevronLeft className="h-4 w-4" />
    <span className="sr-only">Zurück</span>
  </PaginationLink>
)
PaginationPrevious.displayName = 'PaginationPrevious'

const PaginationNext = ({ className, ...props }: React.ComponentProps<typeof PaginationLink>) => (
  <PaginationLink data-slot="pagination-next" aria-label="Zur nächsten Seite" size="icon" className={className} {...props}>
    <ChevronRight className="h-4 w-4" />
    <span className="sr-only">Weiter</span>
  </PaginationLink>
)
PaginationNext.displayName = 'PaginationNext'

const PaginationEllipsis = ({ className, ...props }: React.ComponentProps<'span'>) => (
  <span
    aria-hidden
    data-slot="pagination-ellipsis"
    className={cn('flex h-9 w-9 items-center justify-center text-dark-400', className)}
    {...props}
  >
    <MoreHorizontal className="h-4 w-4" />
    <span className="sr-only">Weitere Seiten</span>
  </span>
)
PaginationEllipsis.displayName = 'PaginationEllipsis'

export interface PaginationData {
  currentPage: number
  totalPages: number
  hasNextPage: boolean
  hasPreviousPage: boolean
}

export interface SimplePaginationProps {
  pagination: PaginationData
  onPageChange: (page: number) => void
  className?: string
}

function SimplePagination({ pagination, onPageChange, className }: SimplePaginationProps) {
  const { currentPage, totalPages, hasNextPage, hasPreviousPage } = pagination

  const getPageNumbers = () => {
    const pages: (number | 'ellipsis')[] = []
    const maxVisible = 5

    if (totalPages <= maxVisible) {
      for (let i = 1; i <= totalPages; i++) {
        pages.push(i)
      }
    } else {
      pages.push(1)

      if (currentPage > 3) {
        pages.push('ellipsis')
      }

      const start = Math.max(2, currentPage - 1)
      const end = Math.min(totalPages - 1, currentPage + 1)

      for (let i = start; i <= end; i++) {
        pages.push(i)
      }

      if (currentPage < totalPages - 2) {
        pages.push('ellipsis')
      }

      pages.push(totalPages)
    }

    return pages
  }

  return (
    <Pagination className={className}>
      <PaginationContent>
        <PaginationItem>
          <Button
            variant="ghost"
            size="icon"
            onClick={() => onPageChange(currentPage - 1)}
            disabled={!hasPreviousPage}
            aria-label="Vorherige Seite"
          >
            <ChevronLeft className="h-4 w-4" />
          </Button>
        </PaginationItem>

        {getPageNumbers().map((page, index) =>
          page === 'ellipsis' ? (
            <PaginationItem key={`ellipsis-${index}`}>
              <PaginationEllipsis />
            </PaginationItem>
          ) : (
            <PaginationItem key={page}>
              <PaginationLink
                href="#"
                onClick={(e) => {
                  e.preventDefault()
                  onPageChange(page)
                }}
                isActive={page === currentPage}
              >
                {page}
              </PaginationLink>
            </PaginationItem>
          ),
        )}

        <PaginationItem>
          <Button
            variant="ghost"
            size="icon"
            onClick={() => onPageChange(currentPage + 1)}
            disabled={!hasNextPage}
            aria-label="Nächste Seite"
          >
            <ChevronRight className="h-4 w-4" />
          </Button>
        </PaginationItem>
      </PaginationContent>
    </Pagination>
  )
}

export {
  Pagination,
  PaginationContent,
  PaginationEllipsis,
  PaginationItem,
  PaginationLink,
  PaginationNext,
  PaginationPrevious,
  SimplePagination,
}
