import * as React from 'react'
import { DayPicker } from 'react-day-picker'
import { ChevronDown, ChevronLeft, ChevronRight } from 'lucide-react'
import { setMonth, setYear } from 'date-fns'

import { cn } from '@/lib/utils'
import { buttonVariants } from '@/components/ui/button'

type CalendarProps = React.ComponentProps<typeof DayPicker>

const MONTHS_DE = [
  'Januar',
  'Februar',
  'März',
  'April',
  'Mai',
  'Juni',
  'Juli',
  'August',
  'September',
  'Oktober',
  'November',
  'Dezember',
]

function CalendarHeader({
  displayMonth,
  onMonthChange,
  onPreviousMonth,
  onNextMonth,
}: {
  displayMonth: Date
  onMonthChange: (month: Date) => void
  onPreviousMonth: () => void
  onNextMonth: () => void
}) {
  const currentYear = new Date().getFullYear()
  const years = Array.from({ length: 21 }, (_, i) => currentYear - 10 + i)

  return (
    <div className="flex items-center justify-between px-1 pt-1">
      <button
        type="button"
        onClick={onPreviousMonth}
        className={cn(
          buttonVariants({ variant: 'ghost', size: 'icon' }),
          'size-8 p-0 opacity-60 hover:opacity-100 [&_svg]:size-4',
        )}
        aria-label="Vorheriger Monat"
      >
        <ChevronLeft className="size-4" />
      </button>

      <div className="flex items-center gap-1">
        <div className="relative">
          <select
            value={displayMonth.getMonth()}
            onChange={(e) => onMonthChange(setMonth(displayMonth, parseInt(e.target.value)))}
            className="h-8 appearance-none rounded-md bg-transparent py-1 pl-2 pr-6 text-sm font-medium outline-none hover:bg-accent cursor-pointer transition-colors focus-visible:ring-green-dark/50 focus-visible:ring-[3px]"
            aria-label="Monat auswählen"
          >
            {MONTHS_DE.map((month, i) => (
              <option key={month} value={i}>
                {month}
              </option>
            ))}
          </select>
          <ChevronDown className="pointer-events-none absolute right-1 top-1/2 size-3.5 -translate-y-1/2 opacity-50" />
        </div>
        <div className="relative">
          <select
            value={displayMonth.getFullYear()}
            onChange={(e) => onMonthChange(setYear(displayMonth, parseInt(e.target.value)))}
            className="h-8 appearance-none rounded-md bg-transparent py-1 pl-2 pr-6 text-sm font-medium outline-none hover:bg-accent cursor-pointer transition-colors focus-visible:ring-green-dark/50 focus-visible:ring-[3px]"
            aria-label="Jahr auswählen"
          >
            {years.map((year) => (
              <option key={year} value={year}>
                {year}
              </option>
            ))}
          </select>
          <ChevronDown className="pointer-events-none absolute right-1 top-1/2 size-3.5 -translate-y-1/2 opacity-50" />
        </div>
      </div>

      <button
        type="button"
        onClick={onNextMonth}
        className={cn(
          buttonVariants({ variant: 'ghost', size: 'icon' }),
          'size-8 p-0 opacity-60 hover:opacity-100 [&_svg]:size-4',
        )}
        aria-label="Nächster Monat"
      >
        <ChevronRight className="size-4" />
      </button>
    </div>
  )
}

function Calendar({ className, classNames, showOutsideDays = true, ...props }: CalendarProps) {
  const [displayMonth, setDisplayMonth] = React.useState<Date>(
    () => (props as { selected?: Date }).selected ?? new Date(),
  )

  const handlePreviousMonth = () => {
    setDisplayMonth((prev) => {
      const d = new Date(prev)
      d.setMonth(d.getMonth() - 1)
      return d
    })
  }

  const handleNextMonth = () => {
    setDisplayMonth((prev) => {
      const d = new Date(prev)
      d.setMonth(d.getMonth() + 1)
      return d
    })
  }

  return (
    <div className={cn('p-3', className)}>
      <CalendarHeader
        displayMonth={displayMonth}
        onMonthChange={setDisplayMonth}
        onPreviousMonth={handlePreviousMonth}
        onNextMonth={handleNextMonth}
      />
      <DayPicker
        showOutsideDays={showOutsideDays}
        month={displayMonth}
        onMonthChange={setDisplayMonth}
        hideNavigation
        classNames={{
          months: 'flex flex-col sm:flex-row gap-2',
          month: 'flex flex-col gap-4',
          month_caption: 'hidden',
          month_grid: 'w-full border-collapse space-x-1',
          weekdays: 'flex',
          weekday: 'text-muted-foreground rounded-md w-9 font-normal text-[0.8rem]',
          week: 'flex w-full mt-2',
          day: cn(
            'relative p-0 text-center text-sm focus-within:relative focus-within:z-20 [&:has([aria-selected].day-outside)]:bg-accent/50',
            props.mode === 'range'
              ? '[&:has([aria-selected])]:bg-accent [&:has(>.day-range-end)]:rounded-r-full [&:has(>.day-range-start)]:rounded-l-full first:[&:has([aria-selected])]:rounded-l-full last:[&:has([aria-selected])]:rounded-r-full [&:has([aria-selected].day-range-end)]:rounded-r-full'
              : '[&:has([aria-selected])]:rounded-full',
          ),
          day_button: cn(
            buttonVariants({ variant: 'ghost' }),
            'size-9 rounded-full p-0 font-normal aria-selected:opacity-100',
          ),
          range_end: 'day-range-end',
          range_start: 'day-range-start',
          selected:
            'rounded-full bg-primary text-primary-foreground hover:bg-primary hover:text-primary-foreground focus:bg-primary focus:text-primary-foreground',
          today: 'rounded-full bg-accent text-accent-foreground',
          outside:
            'day-outside text-muted-foreground aria-selected:text-muted-foreground opacity-50',
          disabled: 'text-muted-foreground opacity-50',
          range_middle: 'aria-selected:bg-accent aria-selected:text-accent-foreground',
          hidden: 'invisible',
          ...classNames,
        }}
        {...props}
      />
    </div>
  )
}
Calendar.displayName = 'Calendar'

export { Calendar }
export type { CalendarProps }
