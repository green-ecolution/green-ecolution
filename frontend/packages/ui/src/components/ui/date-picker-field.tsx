import * as React from 'react'
import { format } from 'date-fns'
import { de } from 'date-fns/locale'
import { CalendarIcon } from 'lucide-react'

import { cn } from '@/lib/utils'
import { Calendar } from '@/components/ui/calendar'
import { Label } from '@/components/ui/label'
import { Popover, PopoverContent, PopoverTrigger } from '@/components/ui/popover'

export interface DatePickerFieldProps {
  label: string
  value?: Date
  onChange?: (date: Date | undefined) => void
  error?: string
  description?: string
  hideLabel?: boolean
  required?: boolean
  disabled?: boolean
  id?: string
  className?: string
  placeholder?: string
  /** Earliest selectable date */
  fromDate?: Date
  /** Latest selectable date */
  toDate?: Date
}

function DatePickerField({
  label,
  value,
  onChange,
  error,
  description,
  hideLabel,
  required,
  disabled,
  id,
  className,
  placeholder = 'Datum auswählen',
  fromDate,
  toDate,
}: DatePickerFieldProps) {
  const inputId = id || React.useId()
  const [open, setOpen] = React.useState(false)

  return (
    <div data-slot="date-picker-field" className={cn('flex flex-col gap-y-2', className)}>
      <Label htmlFor={inputId} className={cn(hideLabel && 'sr-only', error && 'text-destructive')}>
        {label}
        {required && <span className="text-destructive ml-1">*</span>}
      </Label>
      <Popover open={open} onOpenChange={setOpen}>
        <PopoverTrigger asChild>
          <button
            type="button"
            id={inputId}
            disabled={disabled}
            aria-invalid={!!error}
            aria-describedby={
              error ? `${inputId}-error` : description ? `${inputId}-description` : undefined
            }
            className={cn(
              'flex h-10 w-full items-center gap-2 rounded-lg border border-dark-200 bg-white px-3 py-2 text-base text-dark-800 shadow-xs transition-[color,box-shadow] outline-none cursor-pointer focus-visible:border-green-dark focus-visible:ring-green-dark/50 focus-visible:ring-[3px] disabled:cursor-not-allowed disabled:opacity-50 md:text-sm',
              !value && 'text-dark-400',
              error && 'border-destructive focus-visible:ring-destructive/50',
            )}
          >
            <CalendarIcon className="size-4 shrink-0 opacity-50" />
            <span className="flex-1 text-left">
              {value ? format(value, 'PPP', { locale: de }) : placeholder}
            </span>
          </button>
        </PopoverTrigger>
        <PopoverContent className="w-auto p-0" align="start">
          <Calendar
            mode="single"
            selected={value}
            onSelect={(date) => {
              onChange?.(date)
              setOpen(false)
            }}
            locale={de}
            disabled={[
              ...(fromDate ? [{ before: fromDate }] : []),
              ...(toDate ? [{ after: toDate }] : []),
            ]}
            defaultMonth={value}
          />
        </PopoverContent>
      </Popover>
      {description && !error && (
        <p
          id={`${inputId}-description`}
          data-slot="date-picker-field-description"
          className="text-sm text-muted-foreground"
        >
          {description}
        </p>
      )}
      {error && (
        <p
          id={`${inputId}-error`}
          data-slot="date-picker-field-error"
          className="text-sm text-destructive"
        >
          {error}
        </p>
      )}
    </div>
  )
}
DatePickerField.displayName = 'DatePickerField'

export { DatePickerField }
