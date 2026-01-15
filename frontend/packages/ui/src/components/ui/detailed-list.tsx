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
}

const DetailedList = React.forwardRef<HTMLDListElement, DetailedListProps>(
  ({ className, headline, details, columns = 2, ...props }, ref) => {
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
            'space-y-3',
            columns === 2 && 'grid grid-cols-1 gap-x-6 gap-y-3 sm:grid-cols-2',
            className,
          )}
          {...props}
        >
          {details.map((detail, index) => (
            <div key={index} data-slot="detailed-list-item" className="flex flex-col">
              <dt data-slot="detailed-list-label" className="text-sm font-medium text-muted-foreground">
                {detail.label}
              </dt>
              <dd data-slot="detailed-list-value" className="mt-1 text-sm text-foreground">
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
