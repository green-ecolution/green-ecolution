import { Button } from '@/components/ui/button'
import { cn } from '@/lib/utils'

export interface TimeRangeToggleOption<T extends string> {
  value: T
  label: string
}

export interface TimeRangeToggleProps<T extends string> {
  options: TimeRangeToggleOption<T>[]
  value: T
  onChange: (value: T) => void
  ariaLabel?: string
  className?: string
}

export const TimeRangeToggle = <T extends string>({
  options,
  value,
  onChange,
  ariaLabel = 'Zeitraum',
  className,
}: TimeRangeToggleProps<T>) => (
  <div role="group" aria-label={ariaLabel} className={cn('flex items-center gap-1', className)}>
    {options.map((option) => (
      <Button
        key={option.value}
        type="button"
        size="sm"
        variant={option.value === value ? 'default' : 'ghost'}
        aria-pressed={option.value === value}
        onClick={() => onChange(option.value)}
      >
        {option.label}
      </Button>
    ))}
  </div>
)
