import type { ComponentType } from 'react'
import { Check, ChevronDown } from 'lucide-react'
import {
  Badge,
  Button,
  cn,
  Command,
  CommandEmpty,
  CommandGroup,
  CommandItem,
  CommandList,
  Popover,
  PopoverContent,
  PopoverTrigger,
} from '@green-ecolution/ui'

export interface FilterOption {
  value: string
  label: string
}

interface ListFilterDropdownProps {
  label: string
  options: FilterOption[]
  value: string[]
  onChange: (value: string[]) => void
  icon?: ComponentType<{ className?: string }>
  emptyText: string
}

const ListFilterDropdown = ({
  label,
  options,
  value,
  onChange,
  icon: Icon,
  emptyText,
}: ListFilterDropdownProps) => {
  const toggle = (option: string) =>
    onChange(value.includes(option) ? value.filter((v) => v !== option) : [...value, option])

  return (
    <Popover>
      <PopoverTrigger asChild>
        <Button
          variant="outline"
          className={cn('gap-2', value.length > 0 && 'border-green-dark text-green-dark')}
        >
          {Icon && <Icon aria-hidden className="size-4" />}
          {label}
          {value.length > 0 && (
            <Badge variant="green-dark" className="px-1.5">
              {value.length}
            </Badge>
          )}
          <ChevronDown aria-hidden className="size-3.5" />
        </Button>
      </PopoverTrigger>
      <PopoverContent align="start" className="w-64 p-0">
        <Command>
          <CommandList>
            <CommandEmpty>{emptyText}</CommandEmpty>
            <CommandGroup>
              {options.map((option) => (
                <CommandItem
                  key={option.value}
                  value={option.label}
                  onSelect={() => toggle(option.value)}
                >
                  <Check
                    aria-hidden
                    className={cn(
                      'mr-2 size-4',
                      value.includes(option.value) ? 'opacity-100' : 'opacity-0',
                    )}
                  />
                  {option.label}
                </CommandItem>
              ))}
            </CommandGroup>
          </CommandList>
        </Command>
      </PopoverContent>
    </Popover>
  )
}

export default ListFilterDropdown
