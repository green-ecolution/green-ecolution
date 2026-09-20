import { cn } from '@/lib/utils'

export interface SegmentedControlOption<T extends string> {
  value: T
  label: string
}

export interface SegmentedControlProps<T extends string> {
  options: SegmentedControlOption<T>[]
  /** null renders every option unselected — used for mixed or indeterminate state. */
  value: T | null
  onChange: (value: T) => void
  ariaLabel: string
  size?: 'default' | 'sm'
  /** `dark` inverts track and pill for use on the dark surfaces (sidebar, user menu). */
  tone?: 'light' | 'dark'
  disabled?: boolean
  className?: string
}

const toneStyles = {
  light: {
    track: 'bg-dark-100',
    selected: 'bg-white font-semibold text-dark shadow-sm',
    unselected: 'text-dark-600 hover:text-dark',
  },
  dark: {
    track: 'bg-white/10',
    selected: 'bg-light font-semibold text-dark shadow-sm',
    unselected: 'text-light/70 hover:text-light',
  },
} as const

export const SegmentedControl = <T extends string>({
  options,
  value,
  onChange,
  ariaLabel,
  size = 'default',
  tone = 'light',
  disabled = false,
  className,
}: SegmentedControlProps<T>) => {
  const styles = toneStyles[tone]

  const shift = (offset: number) => {
    if (options.length === 0) return
    const current = options.findIndex((option) => option.value === value)
    const base = current === -1 ? 0 : current
    const next = options[(base + offset + options.length) % options.length]
    onChange(next.value)
  }

  const handleKeyDown = (event: React.KeyboardEvent<HTMLButtonElement>) => {
    if (disabled) return
    const offset =
      event.key === 'ArrowRight' || event.key === 'ArrowDown'
        ? 1
        : event.key === 'ArrowLeft' || event.key === 'ArrowUp'
          ? -1
          : 0
    if (offset === 0) return
    event.preventDefault()
    // Claim the arrow keys: inside a Radix menu the same event would otherwise
    // also move the menu highlight. Other keys still bubble, so Escape closes.
    event.stopPropagation()
    shift(offset)
  }

  return (
    <div
      role="radiogroup"
      aria-label={ariaLabel}
      className={cn('inline-flex items-center gap-0.5 rounded-lg p-0.5', styles.track, className)}
    >
      {options.map((option, index) => {
        const selected = option.value === value
        return (
          <button
            key={option.value}
            type="button"
            role="radio"
            aria-checked={selected}
            disabled={disabled}
            tabIndex={selected || (value === null && index === 0) ? 0 : -1}
            onClick={() => onChange(option.value)}
            onKeyDown={handleKeyDown}
            className={cn(
              'rounded-md font-nunito-sans whitespace-nowrap transition-colors',
              'focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring',
              'disabled:cursor-not-allowed disabled:opacity-50',
              size === 'sm' ? 'px-2.5 py-1 text-xs' : 'px-3 py-1.5 text-sm',
              selected ? styles.selected : styles.unselected,
            )}
          >
            {option.label}
          </button>
        )
      })}
    </div>
  )
}
