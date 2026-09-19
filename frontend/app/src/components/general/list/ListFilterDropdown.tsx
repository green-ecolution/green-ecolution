import { useId, type ComponentType } from 'react'
import { Check, ChevronDown } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import {
  Badge,
  Button,
  cn,
  Command,
  CommandEmpty,
  CommandGroup,
  CommandInput,
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

/** Above this many options the list stops being scannable and gets its own search field. */
const SEARCHABLE_FROM = 8

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
  const { t } = useTranslation('common')
  const stateIdPrefix = useId()
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
          {options.length >= SEARCHABLE_FROM && (
            <CommandInput
              placeholder={t('list.filterSearch')}
              aria-label={t('list.filterSearch')}
            />
          )}
          <CommandList>
            <CommandEmpty>{emptyText}</CommandEmpty>
            <CommandGroup>
              {options.map((option) => {
                const selected = value.includes(option.value)
                const stateId = `${stateIdPrefix}-${option.value}`
                return (
                  <CommandItem
                    key={option.value}
                    value={option.label}
                    onSelect={() => toggle(option.value)}
                    // cmdk forces role="option", which the ARIA spec gives no
                    // supported "checked" state; aria-label pins the accessible
                    // name to the plain label so the state text (reachable via
                    // aria-describedby) can't merge into it and can't be
                    // stripped by the check icon's aria-hidden.
                    aria-label={option.label}
                    aria-describedby={stateId}
                  >
                    <Check
                      aria-hidden
                      className={cn('mr-2 size-4', selected ? 'opacity-100' : 'opacity-0')}
                    />
                    {option.label}
                    <span id={stateId} className="sr-only">
                      {selected
                        ? t('list.filterOptionSelected')
                        : t('list.filterOptionNotSelected')}
                    </span>
                  </CommandItem>
                )
              })}
            </CommandGroup>
          </CommandList>
        </Command>
      </PopoverContent>
    </Popover>
  )
}

export default ListFilterDropdown
