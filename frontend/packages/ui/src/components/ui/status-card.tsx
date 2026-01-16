import * as React from 'react'
import { cva, type VariantProps } from 'class-variance-authority'
import { cn } from '@/lib/utils'
import { Badge, type BadgeProps } from './badge'

const statusCardVariants = cva('h-full space-y-3 rounded-xl p-6', {
  variants: {
    status: {
      // Base variants
      default: 'bg-dark-50',
      red: 'bg-red-100',
      yellow: 'bg-yellow-100',
      'green-dark': 'bg-green-dark-100',
      'green-light': 'bg-green-light-100',
      // Badge variant aliases (outline-*)
      'outline-dark': 'bg-dark-50',
      'outline-red': 'bg-red-100',
      'outline-yellow': 'bg-yellow-100',
      'outline-green-dark': 'bg-green-dark-100',
      'outline-green-light': 'bg-green-light-100',
      // Additional Badge variants
      secondary: 'bg-dark-50',
      destructive: 'bg-red-100',
      outline: 'bg-dark-50',
      success: 'bg-green-dark-100',
      warning: 'bg-yellow-100',
      error: 'bg-red-100',
      muted: 'bg-dark-50',
    },
  },
  defaultVariants: {
    status: 'default',
  },
})

const dotVariants = cva('absolute w-4 h-4 rounded-full left-0 top-1', {
  variants: {
    status: {
      // Base variants
      default: 'bg-dark-400',
      red: 'bg-red',
      yellow: 'bg-yellow',
      'green-dark': 'bg-green-dark',
      'green-light': 'bg-green-light',
      // Badge variant aliases (outline-*)
      'outline-dark': 'bg-dark-400',
      'outline-red': 'bg-red',
      'outline-yellow': 'bg-yellow',
      'outline-green-dark': 'bg-green-dark',
      'outline-green-light': 'bg-green-light',
      // Additional Badge variants
      secondary: 'bg-dark-400',
      destructive: 'bg-red',
      outline: 'bg-dark-400',
      success: 'bg-green-dark',
      warning: 'bg-yellow',
      error: 'bg-red',
      muted: 'bg-dark-400',
    },
  },
  defaultVariants: {
    status: 'default',
  },
})

type StatusVariant = NonNullable<VariantProps<typeof statusCardVariants>['status']>

const statusToBadgeVariant: Record<StatusVariant, BadgeProps['variant']> = {
  default: 'outline-dark',
  red: 'outline-red',
  yellow: 'outline-yellow',
  'green-dark': 'outline-green-dark',
  'green-light': 'outline-green-light',
  'outline-dark': 'outline-dark',
  'outline-red': 'outline-red',
  'outline-yellow': 'outline-yellow',
  'outline-green-dark': 'outline-green-dark',
  'outline-green-light': 'outline-green-light',
  secondary: 'secondary',
  destructive: 'destructive',
  outline: 'outline',
  success: 'success',
  warning: 'warning',
  error: 'error',
  muted: 'muted',
}

interface StatusCardProps
  extends React.HTMLAttributes<HTMLDivElement>,
    VariantProps<typeof statusCardVariants> {
  label: string
  value: string | number
  description?: string
  indicator?: 'dot' | 'badge' | 'none'
  isLarge?: boolean
}

const StatusCard = React.forwardRef<HTMLDivElement, StatusCardProps>(
  (
    { className, status = 'default', label, value, description, indicator = 'none', isLarge = false, ...props },
    ref
  ) => {
    const showDot = indicator === 'dot'
    const showBadge = indicator === 'badge'
    const badgeVariant = statusToBadgeVariant[status ?? 'default']

    return (
      <div ref={ref} className={cn(statusCardVariants({ status }), className)} {...props}>
        <p className="text-sm text-dark-700 font-medium">{label}</p>
        <p className={cn('font-bold', isLarge ? 'text-3xl' : 'text-xl', showDot && 'pl-7 relative')}>
          {showDot && <span className={dotVariants({ status })} />}
          {showBadge ? (
            <>
              <Badge variant={badgeVariant} size="lg">
                {value}
              </Badge>
              <span className="sr-only">{value}</span>
            </>
          ) : (
            value
          )}
        </p>
        {description && <p className="text-sm">{description}</p>}
      </div>
    )
  }
)
StatusCard.displayName = 'StatusCard'

export { StatusCard, statusCardVariants }
export type { StatusCardProps, StatusVariant }
