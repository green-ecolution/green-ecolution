import * as React from 'react'
import { X } from 'lucide-react'

import { cn } from '@/lib/utils'
import { useUiText } from '@/i18n'
import { Badge } from './badge'

export interface FilterChipProps extends React.HTMLAttributes<HTMLSpanElement> {
  label: string
  onRemove: () => void
}

const FilterChip = React.forwardRef<HTMLSpanElement, FilterChipProps>(
  ({ label, onRemove, className, ...props }, ref) => {
    const { t } = useUiText()

    return (
      <Badge
        ref={ref}
        variant="outline-green-dark"
        className={cn('gap-1.5 bg-green-dark-50 pr-1.5', className)}
        {...props}
      >
        {label}
        <button
          type="button"
          onClick={onRemove}
          aria-label={t('filterChip.remove', { label })}
          className="rounded-full p-0.5 text-dark-600 transition-colors hover:text-green-dark focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
        >
          <X aria-hidden className="size-3" />
        </button>
      </Badge>
    )
  },
)
FilterChip.displayName = 'FilterChip'

export { FilterChip }
