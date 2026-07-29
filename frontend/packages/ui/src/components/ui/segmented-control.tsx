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
  disabled?: boolean
  className?: string
}

export const SegmentedControl = <T extends string>({
  options,
  value,
  onChange,
  ariaLabel,
  size = 'default',
  disabled = false,
  className,
}: SegmentedControlProps<T>) => {
  const shift = (offset: number) => {
    if (options.length === 0) return
    const current = options.findIndex((option) => option.value === value)
    const base = current === -1 ? 0 : current
    const next = options[(base + offset + options.length) % options.length]
    onChange(next.value)
  }

  const handleKeyDown = (event: React.KeyboardEvent<HTMLDivElement>) => {
    if (disabled) return
    if (event.key === 'ArrowRight' || event.key === 'ArrowDown') {
      event.preventDefault()
      shift(1)
    }
    if (event.key === 'ArrowLeft' || event.key === 'ArrowUp') {
      event.preventDefault()
      shift(-1)
    }
  }

  return (
    <div
      role="radiogroup"
      aria-label={ariaLabel}
      onKeyDown={handleKeyDown}
      className={cn('inline-flex items-center gap-0.5 rounded-lg bg-dark-100 p-0.5', className)}
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
            className={cn(
              'rounded-md font-nunito-sans whitespace-nowrap transition-colors',
              'focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring',
              'disabled:cursor-not-allowed disabled:opacity-50',
              size === 'sm' ? 'px-2.5 py-1 text-xs' : 'px-3 py-1.5 text-sm',
              selected
                ? 'bg-white font-semibold text-dark shadow-sm'
                : 'text-dark-600 hover:text-dark',
            )}
          >
            {option.label}
          </button>
        )
      })}
    </div>
  )
}
