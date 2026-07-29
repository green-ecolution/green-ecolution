import { SegmentedControl, type SegmentedControlOption } from '@/components/ui/segmented-control'

export type TimeRangeToggleOption<T extends string> = SegmentedControlOption<T>

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
  <SegmentedControl
    options={options}
    value={value}
    onChange={onChange}
    ariaLabel={ariaLabel}
    size="sm"
    className={className}
  />
)
