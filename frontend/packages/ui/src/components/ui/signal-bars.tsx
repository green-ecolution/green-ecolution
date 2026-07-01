import { cn } from '@/lib/utils'

export interface SignalBarsProps {
  /** Number of filled bars, 0–4. */
  filled: number
  /** Pulse the leading filled bar (e.g. live tracking / good signal). */
  live?: boolean
  /** Thickness/height preset. `sm` (default) is the inline badge size. */
  size?: 'sm' | 'md' | 'lg'
  className?: string
}

const BAR_HEIGHTS = ['h-[30%]', 'h-[55%]', 'h-[80%]', 'h-full'] as const

const ROOT_SIZE: Record<NonNullable<SignalBarsProps['size']>, string> = {
  sm: 'h-3.5 gap-[2px]',
  md: 'h-6 gap-[3px]',
  lg: 'h-10 gap-1',
}

const BAR_SIZE: Record<NonNullable<SignalBarsProps['size']>, string> = {
  sm: 'w-[3px] rounded-[1px]',
  md: 'w-[5px] rounded-sm',
  lg: 'w-2 rounded-sm',
}

/** Four ascending bars. Bars use `bg-current`, so the consumer sets colour via
 *  a `text-*` class on this element or an ancestor. */
export const SignalBars = ({ filled, live = false, size = 'sm', className }: SignalBarsProps) => (
  <span aria-hidden className={cn('flex items-end', ROOT_SIZE[size], className)}>
    {BAR_HEIGHTS.map((h, i) => {
      const isFilled = i < filled
      const isLeading = i === filled - 1
      return (
        <span
          key={i}
          className={cn(
            'bg-current',
            BAR_SIZE[size],
            h,
            isFilled ? 'opacity-90' : 'opacity-25',
            isFilled && isLeading && live && 'animate-pulse',
          )}
        />
      )
    })}
  </span>
)
