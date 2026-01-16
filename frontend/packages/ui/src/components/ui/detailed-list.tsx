import * as React from 'react'

import { cn } from '@/lib/utils'

export interface DetailItem {
  label: string
  value: React.ReactNode
}

export interface DetailedListProps extends React.HTMLAttributes<HTMLDListElement> {
  headline?: string
  details: DetailItem[]
  columns?: 1 | 2
  variant?: 'default' | 'horizontal'
}

const DetailedList = React.forwardRef<HTMLDListElement, DetailedListProps>(
  ({ className, headline, details, columns = 2, variant = 'default', ...props }, ref) => {
    return (
      <div data-slot="detailed-list">
        {headline && (
          <h3 data-slot="detailed-list-headline" className="mb-4 text-lg font-semibold font-lato">
            {headline}
          </h3>
        )}
        <dl
          ref={ref}
          data-slot="detailed-list-content"
          className={cn(
            variant === 'default' && 'space-y-3',
            variant === 'default' &&
              columns === 2 &&
              'grid grid-cols-1 gap-x-6 gap-y-3 sm:grid-cols-2',
            variant === 'horizontal' && columns === 2 && 'md:grid md:grid-cols-2 md:gap-x-11',
            className,
          )}
          {...props}
        >
          {details.map((detail, index) => (
            <div
              key={index}
              data-slot="detailed-list-item"
              className={cn(
                variant === 'default' && 'flex flex-col',
                variant === 'horizontal' && 'py-4 border-b border-b-dark-200 first:pt-0',
              )}
            >
              <dt
                data-slot="detailed-list-label"
                className={cn(
                  variant === 'default' && 'text-sm font-medium text-muted-foreground',
                  variant === 'horizontal' && 'font-bold inline',
                )}
              >
                {detail.label}
              </dt>
              <dd
                data-slot="detailed-list-value"
                className={cn(
                  variant === 'default' && 'mt-1 text-sm text-foreground',
                  variant === 'horizontal' && 'sm:inline sm:px-2',
                )}
              >
                {detail.value}
              </dd>
            </div>
          ))}
        </dl>
      </div>
    )
  },
)
DetailedList.displayName = 'DetailedList'

export { DetailedList }
