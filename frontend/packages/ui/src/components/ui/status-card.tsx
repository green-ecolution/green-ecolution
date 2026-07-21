import * as React from 'react'
import { cva, type VariantProps } from 'class-variance-authority'
import { Info } from 'lucide-react'
import { cn } from '@/lib/utils'
import { Badge, type BadgeProps } from './badge'
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from './tooltip'

const statusCardVariants = cva('h-full flex flex-col rounded-xl', {
  variants: {
    size: {
      default: 'gap-y-3 p-6',
      compact: 'gap-y-1.5 p-4',
    },
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
    size: 'default',
    status: 'default',
  },
})

const dotVariants = cva('w-4 h-4 rounded-full', {
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

const progressFillVariants = cva('block h-full rounded-full transition-[width]', {
  variants: {
    status: {
      default: 'bg-dark-400',
      red: 'bg-red',
      yellow: 'bg-yellow',
      'green-dark': 'bg-green-dark',
      'green-light': 'bg-green-light',
      'outline-dark': 'bg-dark-400',
      'outline-red': 'bg-red',
      'outline-yellow': 'bg-yellow',
      'outline-green-dark': 'bg-green-dark',
      'outline-green-light': 'bg-green-light',
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
  extends React.HTMLAttributes<HTMLDivElement>, VariantProps<typeof statusCardVariants> {
  label: string
  value: string | number
  description?: React.ReactNode
  indicator?: 'dot' | 'badge' | 'none'
  isLarge?: boolean
  icon?: React.ReactNode
  /** Renders a progress bar (0–100) below the value, colored to match `status`. */
  progress?: number
  /** Renders an info icon next to the label that reveals this text in a tooltip. */
  info?: React.ReactNode
}

const StatusCard = React.forwardRef<HTMLDivElement, StatusCardProps>(
  (
    {
      className,
      size = 'default',
      status = 'default',
      label,
      value,
      description,
      indicator = 'none',
      isLarge = false,
      icon,
      progress,
      info,
      ...props
    },
    ref,
  ) => {
    const showDot = indicator === 'dot' && !icon
    const showIcon = indicator === 'dot' && icon
    const showBadge = indicator === 'badge'
    const badgeVariant = statusToBadgeVariant[status ?? 'default']

    return (
      <div ref={ref} className={cn(statusCardVariants({ size, status }), className)} {...props}>
        <p className="text-sm text-dark-700 font-medium">
          {label}
          {info && (
            <TooltipProvider>
              <Tooltip>
                <TooltipTrigger asChild>
                  <button
                    type="button"
                    aria-label="Weitere Informationen"
                    className="ml-1.5 inline-flex translate-y-0.5 text-dark-500 hover:text-dark-700 focus-visible:outline-2 focus-visible:outline-offset-2"
                  >
                    <Info className="size-3.5" />
                  </button>
                </TooltipTrigger>
                <TooltipContent className="max-w-64">{info}</TooltipContent>
              </Tooltip>
            </TooltipProvider>
          )}
        </p>
        <div className={cn('font-bold flex items-start gap-2', isLarge ? 'text-3xl' : 'text-xl')}>
          {showDot && <span className={cn(dotVariants({ status }), 'mt-1.5 shrink-0')} />}
          {showIcon && <span className="mt-0.5 shrink-0 [&>svg]:size-5">{icon}</span>}
          {showBadge ? (
            <>
              <Badge variant={badgeVariant} size="lg">
                {value}
              </Badge>
              <span className="sr-only">{value}</span>
            </>
          ) : (
            <span>{value}</span>
          )}
        </div>
        {progress != null && (
          <span className="block h-1.5 w-full overflow-hidden rounded-full bg-dark-200">
            <span
              className={progressFillVariants({ status })}
              style={{ width: `${Math.max(0, Math.min(100, progress))}%` }}
            />
          </span>
        )}
        {description && <p className="text-sm">{description}</p>}
      </div>
    )
  },
)
StatusCard.displayName = 'StatusCard'

export { StatusCard, statusCardVariants }
export type { StatusCardProps, StatusVariant }
