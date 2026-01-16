import * as React from 'react'

import { cn } from '@/lib/utils'

export interface MultiSelectOption {
  value: string
  label: string
  disabled?: boolean
}

export interface MultiSelectProps
  extends Omit<React.SelectHTMLAttributes<HTMLSelectElement>, 'value' | 'onChange'> {
  value?: string[]
  onChange?: (value: string[]) => void
  options?: MultiSelectOption[]
  placeholder?: string
}

const MultiSelect = React.forwardRef<HTMLSelectElement, MultiSelectProps>(
  ({ className, value, onChange, options, children, ...props }, ref) => {
    const handleChange = (e: React.ChangeEvent<HTMLSelectElement>) => {
      if (onChange) {
        const selectedValues = Array.from(e.target.selectedOptions, (option) => option.value)
        onChange(selectedValues)
      }
    }

    return (
      <select
        ref={ref}
        multiple
        data-slot="multi-select"
        value={value}
        onChange={handleChange}
        className={cn(
          'flex min-h-[120px] w-full rounded-lg border border-dark-200 bg-white px-3 py-2 text-sm text-dark-800 shadow-xs transition-[color,box-shadow] outline-none focus:border-green-dark focus:ring-green-dark/50 focus:ring-[3px] disabled:cursor-not-allowed disabled:opacity-50',
          '[&_option]:py-1.5 [&_option]:px-2 [&_option:checked]:bg-green-light-100',
          className,
        )}
        {...props}
      >
        {options
          ? options.map((option) => (
              <option key={option.value} value={option.value} disabled={option.disabled}>
                {option.label}
              </option>
            ))
          : children}
      </select>
    )
  },
)
MultiSelect.displayName = 'MultiSelect'

export { MultiSelect }
