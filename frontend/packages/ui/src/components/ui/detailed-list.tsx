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
      <div className={className}>
        {headline && <h3 className="mb-4 text-lg font-semibold font-lato">{headline}</h3>}
        <dl
          ref={ref}
          className={cn(
            'space-y-3',
            columns === 2 && 'grid grid-cols-1 gap-x-6 gap-y-3 sm:grid-cols-2',
          )}
          {...props}
        >
          {details.map((detail, index) => (
            <div key={index} className="flex flex-col">
              <dt className="text-sm font-medium text-muted-foreground">{detail.label}</dt>
              <dd className="mt-1 text-sm text-foreground">{detail.value}</dd>
            </div>
          ))}
        </dl>
      </div>
    )
  },
)
DetailedList.displayName = 'DetailedList'

export { DetailedList }
