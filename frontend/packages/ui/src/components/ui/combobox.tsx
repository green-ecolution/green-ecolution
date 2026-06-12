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

export interface ComboboxOption {
  value: string
  label: string
  group?: string
  disabled?: boolean
}

export interface ComboboxProps {
  options: ComboboxOption[]
  value?: string
  onChange?: (value: string) => void
  placeholder?: string
  searchPlaceholder?: string
  emptyText?: string
  id?: string
  disabled?: boolean
  className?: string
}

const Combobox = React.forwardRef<HTMLButtonElement, ComboboxProps>(
  (
    {
      options,
      value,
      onChange,
      placeholder = 'Auswählen…',
      searchPlaceholder = 'Suchen…',
      emptyText = 'Keine Treffer.',
      id,
      disabled,
      className,
    },
    ref,
  ) => {
    const [open, setOpen] = React.useState(false)
    const selected = options.find((o) => o.value === value)

    const groups = React.useMemo(() => {
      const map = new Map<string, ComboboxOption[]>()
      for (const option of options) {
        const key = option.group ?? ''
        const bucket = map.get(key)
        if (bucket) bucket.push(option)
        else map.set(key, [option])
      }
      return Array.from(map, ([label, opts]) => ({ label, options: opts }))
    }, [options])

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
            data-slot="combobox-trigger"
            className={cn(
              'flex h-10 w-full items-center justify-between rounded-lg border border-dark-200 bg-white px-3 py-2 text-sm shadow-xs transition-[color,box-shadow] outline-none focus:border-green-dark focus:ring-green-dark/50 focus:ring-[3px] disabled:cursor-not-allowed disabled:opacity-50',
              selected ? 'text-dark-800' : 'text-dark-400',
              className,
            )}
          >
            <span className="line-clamp-1 text-left">
              {selected ? selected.label : placeholder}
            </span>
            <ChevronsUpDown className="ml-2 h-4 w-4 shrink-0 opacity-50" />
          </button>
        </PopoverTrigger>
        <PopoverContent align="start" className="w-[var(--radix-popover-trigger-width)] p-0">
          <Command>
            <CommandInput placeholder={searchPlaceholder} />
            <CommandList>
              <CommandEmpty>{emptyText}</CommandEmpty>
              {groups.map((group) => (
                <CommandGroup key={group.label || '_ungrouped'} heading={group.label || undefined}>
                  {group.options.map((option) => (
                    <CommandItem
                      key={option.value}
                      value={`${option.value} ${option.label}`}
                      disabled={option.disabled}
                      onSelect={() => {
                        onChange?.(option.value)
                        setOpen(false)
                      }}
                    >
                      <Check
                        className={cn(
                          'h-4 w-4 shrink-0',
                          option.value === value ? 'opacity-100' : 'opacity-0',
                        )}
                      />
                      <span className="line-clamp-1">{option.label}</span>
                    </CommandItem>
                  ))}
                </CommandGroup>
              ))}
            </CommandList>
          </Command>
        </PopoverContent>
      </Popover>
    )
  },
)
Combobox.displayName = 'Combobox'

export { Combobox }
