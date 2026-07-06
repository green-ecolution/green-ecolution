import * as React from 'react'

import { cn } from '@/lib/utils'
import { Label } from '@/components/ui/label'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'

export interface SelectFieldOption {
  value: string
  label: string
}

export interface SelectFieldProps {
  label: string
  options: SelectFieldOption[]
  value?: string
  onValueChange?: (value: string) => void
  error?: string
  description?: string
  hideLabel?: boolean
  required?: boolean
  disabled?: boolean
  id?: string
  className?: string
  placeholder?: string
  contentClassName?: string
}

function SelectField({
  label,
  options,
  value,
  onValueChange,
  error,
  description,
  hideLabel,
  required,
  disabled,
  id,
  className,
  placeholder,
  contentClassName,
}: SelectFieldProps) {
  const generatedId = React.useId()
  const inputId = id ?? generatedId

  return (
    <div data-slot="select-field" className={cn('flex flex-col gap-y-2', className)}>
      <Label htmlFor={inputId} className={cn(hideLabel && 'sr-only', error && 'text-destructive')}>
        {label}
        {required && <span className="text-destructive ml-1">*</span>}
      </Label>
      <Select value={value} onValueChange={onValueChange} disabled={disabled}>
        <SelectTrigger
          id={inputId}
          aria-invalid={!!error}
          aria-describedby={
            error ? `${inputId}-error` : description ? `${inputId}-description` : undefined
          }
          className={cn(error && 'border-destructive focus:ring-destructive/50')}
        >
          <SelectValue placeholder={placeholder} />
        </SelectTrigger>
        <SelectContent className={contentClassName}>
          {options.map((option) => (
            <SelectItem key={option.value} value={option.value}>
              {option.label}
            </SelectItem>
          ))}
        </SelectContent>
      </Select>
      {description && !error && (
        <p
          id={`${inputId}-description`}
          data-slot="select-field-description"
          className="text-sm text-muted-foreground"
        >
          {description}
        </p>
      )}
      {error && (
        <p
          id={`${inputId}-error`}
          data-slot="select-field-error"
          className="text-sm text-destructive"
        >
          {error}
        </p>
      )}
    </div>
  )
}
SelectField.displayName = 'SelectField'

export { SelectField }
