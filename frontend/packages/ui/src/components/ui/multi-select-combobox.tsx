import * as React from 'react'
import { Check, ChevronsUpDown } from 'lucide-react'

import { cn } from '@/lib/utils'
import { Popover, PopoverContent, PopoverTrigger } from './popover'
import {
  Command,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
} from './command'

export interface MultiSelectComboboxOption {
  value: string
  label: string
  group?: string
  disabled?: boolean
}

export interface MultiSelectComboboxProps {
  options: MultiSelectComboboxOption[]
  value: string[]
  onChange: (value: string[]) => void
  placeholder?: string
  searchPlaceholder?: string
  emptyText?: string
  /** Number of selections above which the trigger shows "N ausgewählt" instead of the labels. */
  summaryThreshold?: number
  searchable?: boolean
  id?: string
  disabled?: boolean
  className?: string
}

const MultiSelectCombobox = React.forwardRef<HTMLButtonElement, MultiSelectComboboxProps>(
  (
    {
      options,
      value,
      onChange,
      placeholder = 'Auswählen…',
      searchPlaceholder = 'Suchen…',
      emptyText = 'Keine Treffer.',
      summaryThreshold = 1,
      searchable = true,
      id,
      disabled,
      className,
    },
    ref,
  ) => {
    const [open, setOpen] = React.useState(false)
    const selected = React.useMemo(() => new Set(value), [value])

    const groups = React.useMemo(() => {
      const map = new Map<string, MultiSelectComboboxOption[]>()
      for (const option of options) {
        const key = option.group ?? ''
        const bucket = map.get(key)
        if (bucket) bucket.push(option)
        else map.set(key, [option])
      }
      return Array.from(map, ([label, opts]) => ({ label, options: opts }))
    }, [options])

    const toggle = (val: string) => {
      if (selected.has(val)) onChange(value.filter((v) => v !== val))
      else onChange([...value, val])
    }

    const triggerLabel =
      value.length === 0
        ? placeholder
        : value.length <= summaryThreshold
          ? value.map((v) => options.find((o) => o.value === v)?.label ?? v).join(', ')
          : `${value.length} ausgewählt`

    return (
      <Popover open={open} onOpenChange={setOpen}>
        <PopoverTrigger asChild>
          <button
            ref={ref}
            id={id}
            type="button"
            role="combobox"
            aria-expanded={open}
            disabled={disabled}
            data-slot="multi-select-combobox-trigger"
            className={cn(
              'flex h-10 w-full items-center justify-between rounded-lg border border-dark-200 bg-white px-3 py-2 text-sm shadow-xs transition-[color,box-shadow] outline-none focus:border-green-dark focus:ring-green-dark/50 focus:ring-[3px] disabled:cursor-not-allowed disabled:opacity-50',
              value.length ? 'text-dark-800' : 'text-dark-400',
              className,
            )}
          >
            <span className="line-clamp-1 text-left">{triggerLabel}</span>
            <ChevronsUpDown className="ml-2 h-4 w-4 shrink-0 opacity-50" />
          </button>
        </PopoverTrigger>
        <PopoverContent align="start" className="w-[var(--radix-popover-trigger-width)] p-0">
          <Command>
            {searchable && <CommandInput placeholder={searchPlaceholder} />}
            <CommandList>
              <CommandEmpty>{emptyText}</CommandEmpty>
              {groups.map((group) => (
                <CommandGroup key={group.label || '_ungrouped'} heading={group.label || undefined}>
                  {group.options.map((option) => {
                    const isSelected = selected.has(option.value)
                    return (
                      <CommandItem
                        key={option.value}
                        value={`${option.value} ${option.label}`}
                        disabled={option.disabled}
                        onSelect={() => toggle(option.value)}
                      >
                        <span
                          aria-hidden
                          className={cn(
                            'flex h-4 w-4 shrink-0 items-center justify-center rounded border',
                            isSelected
                              ? 'border-green-dark bg-green-dark text-white'
                              : 'border-dark-300',
                          )}
                        >
                          {isSelected && <Check className="h-3 w-3" />}
                        </span>
                        <span className="line-clamp-1">{option.label}</span>
                      </CommandItem>
                    )
                  })}
                </CommandGroup>
              ))}
            </CommandList>
          </Command>
        </PopoverContent>
      </Popover>
    )
  },
)
MultiSelectCombobox.displayName = 'MultiSelectCombobox'

export { MultiSelectCombobox }
