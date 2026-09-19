import { ArrowDown, ArrowUp, ArrowUpDown } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import {
  Button,
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuLabel,
  DropdownMenuRadioGroup,
  DropdownMenuRadioItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@green-ecolution/ui'

export type SortDirection = 'asc' | 'desc'

export interface SortOption<T extends string = string> {
  value: T
  label: string
}

interface ListSortMenuProps<T extends string> {
  options: SortOption<T>[]
  field: T
  direction: SortDirection
  onChange: (field: T, direction: SortDirection) => void
}

const ListSortMenu = <T extends string>({
  options,
  field,
  direction,
  onChange,
}: ListSortMenuProps<T>) => {
  const { t } = useTranslation('common')
  const active = options.find((option) => option.value === field)
  const DirectionIcon = direction === 'asc' ? ArrowUp : ArrowDown

  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <Button variant="outline" className="gap-2">
          <ArrowUpDown aria-hidden className="size-4" />
          {active ? active.label : t('list.sortLabel')}
          <DirectionIcon aria-hidden className="size-3.5" />
        </Button>
      </DropdownMenuTrigger>
      <DropdownMenuContent align="end">
        <DropdownMenuLabel>{t('list.sortLabel')}</DropdownMenuLabel>
        <DropdownMenuRadioGroup
          value={field}
          // Radix hands back the raw string value; it is always one of `options`'
          // T-typed values since those are the only items rendered.
          onValueChange={(newField) => onChange(newField as T, direction)}
        >
          {options.map((option) => (
            <DropdownMenuRadioItem key={option.value} value={option.value}>
              {option.label}
            </DropdownMenuRadioItem>
          ))}
        </DropdownMenuRadioGroup>
        <DropdownMenuSeparator />
        <DropdownMenuRadioGroup
          value={direction}
          onValueChange={(newDirection) => onChange(field, newDirection as SortDirection)}
        >
          <DropdownMenuRadioItem value="asc">
            <ArrowUp aria-hidden className="mr-2 size-4" />
            {t('list.sortAscending')}
          </DropdownMenuRadioItem>
          <DropdownMenuRadioItem value="desc">
            <ArrowDown aria-hidden className="mr-2 size-4" />
            {t('list.sortDescending')}
          </DropdownMenuRadioItem>
        </DropdownMenuRadioGroup>
      </DropdownMenuContent>
    </DropdownMenu>
  )
}

export default ListSortMenu
