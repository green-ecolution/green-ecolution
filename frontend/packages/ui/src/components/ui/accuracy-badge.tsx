import * as React from 'react'
import { cva, type VariantProps } from 'class-variance-authority'

import { cn } from '@/lib/utils'

const accuracyBadgeVariants = cva(
  'inline-flex items-center gap-1.5 rounded-full border px-2.5 py-1 text-xs font-semibold leading-none tabular-nums transition-colors',
  {
    variants: {
      level: {
        excellent: 'border-green-dark-200 bg-green-dark-50 text-green-dark',
        good: 'border-green-light-200 bg-green-light-50 text-green-light-900',
        fair: 'border-yellow-200 bg-yellow-50 text-yellow-900',
        poor: 'border-red-200 bg-red-50 text-red',
        searching: 'border-dark-200 bg-dark-50 text-dark-600',
      },
    },
    defaultVariants: {
      level: 'searching',
    },
  },
)

export type AccuracyLevel = NonNullable<VariantProps<typeof accuracyBadgeVariants>['level']>

const LEVEL_LABEL: Record<AccuracyLevel, string> = {
  excellent: 'Sehr gut',
  good: 'Gut',
  fair: 'Mäßig',
  poor: 'Ungenau',
  searching: 'Suche …',
}

export const accuracyLevelFromMeters = (meters: number | null | undefined): AccuracyLevel => {
  if (meters == null || !Number.isFinite(meters) || meters < 0) return 'searching'
  if (meters < 10) return 'excellent'
  if (meters < 30) return 'good'
  if (meters < 75) return 'fair'
  return 'poor'
}

const BAR_COUNT: Record<AccuracyLevel, number> = {
  excellent: 4,
  good: 3,
  fair: 2,
  poor: 1,
  searching: 0,
}

const BAR_HEIGHTS = ['h-[30%]', 'h-[55%]', 'h-[80%]', 'h-full'] as const

const SignalBars = ({ level }: { level: AccuracyLevel }) => {
  const filled = BAR_COUNT[level]
  const isLive = level === 'excellent' || level === 'good'
  return (
    <span aria-hidden className="flex h-3.5 items-end gap-[2px]">
      {BAR_HEIGHTS.map((h, i) => {
        const isFilled = i < filled
        const isLeading = i === filled - 1
        return (
          <span
            key={i}
            className={cn(
              'w-[3px] rounded-[1px] bg-current',
              h,
              isFilled ? 'opacity-90' : 'opacity-25',
              isFilled && isLeading && isLive && 'animate-pulse',
            )}
          />
        )
      })}
    </span>
  )
}

const formatMeters = (m: number): string => {
  if (m >= 100) return `${Math.round(m)} m`
  if (m < 10) return `~${m.toFixed(1)} m`
  return `~${Math.round(m)} m`
}

export interface AccuracyBadgeProps extends Omit<
  React.HTMLAttributes<HTMLSpanElement>,
  'children'
> {
  /** Accuracy in meters (e.g. GeolocationCoordinates.accuracy). */
  accuracyMeters?: number | null
  /** Hide the numeric value — show only level label + bars. */
  hideValue?: boolean
}

const AccuracyBadge = React.forwardRef<HTMLSpanElement, AccuracyBadgeProps>(
  ({ accuracyMeters, hideValue, className, ...rest }, ref) => {
    const level = accuracyLevelFromMeters(accuracyMeters)
    const label = LEVEL_LABEL[level]
    const showValue =
      level !== 'searching' &&
      !hideValue &&
      accuracyMeters != null &&
      Number.isFinite(accuracyMeters)
    const value = showValue ? formatMeters(accuracyMeters as number) : null
    const aria = `GPS-Genauigkeit: ${label}${value ? `, ${value}` : ''}`

    return (
      <span
        ref={ref}
        role="status"
        aria-label={aria}
        className={cn(accuracyBadgeVariants({ level }), className)}
        {...rest}
      >
        <SignalBars level={level} />
        <span>{label}</span>
        {value && <span className="ml-0.5 text-[0.95em] font-normal opacity-70">{value}</span>}
      </span>
    )
  },
)
AccuracyBadge.displayName = 'AccuracyBadge'

export { AccuracyBadge, accuracyBadgeVariants }
