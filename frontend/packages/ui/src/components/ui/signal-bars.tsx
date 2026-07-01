import { cn } from '@/lib/utils'

export interface SignalBarsProps {
  /** Number of filled bars, 0–4. */
  filled: number
  /** Pulse the leading filled bar (e.g. live tracking / good signal). */
  live?: boolean
  className?: string
}

const BAR_HEIGHTS = ['h-[30%]', 'h-[55%]', 'h-[80%]', 'h-full'] as const

/** Four ascending bars. Bars use `bg-current`, so the consumer sets colour via
 *  a `text-*` class on this element or an ancestor. */
export const SignalBars = ({ filled, live = false, className }: SignalBarsProps) => (
  <span aria-hidden className={cn('flex h-3.5 items-end gap-[2px]', className)}>
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
            isFilled && isLeading && live && 'animate-pulse',
          )}
        />
      )
    })}
  </span>
)
