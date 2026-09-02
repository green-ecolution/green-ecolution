import { SegmentedControl, type SegmentedControlOption } from '@/components/ui/segmented-control'
import { useUiText } from '@/i18n'

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
  ariaLabel,
  className,
}: TimeRangeToggleProps<T>) => {
  const { t } = useUiText()
  return (
    <SegmentedControl
      options={options}
      value={value}
      onChange={onChange}
      ariaLabel={ariaLabel ?? t('timeRange.label')}
      size="sm"
      className={className}
    />
  )
}
