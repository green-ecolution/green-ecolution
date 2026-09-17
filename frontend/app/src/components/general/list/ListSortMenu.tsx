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

export interface SortOption {
  value: string
  label: string
}

interface ListSortMenuProps {
  options: SortOption[]
  field: string
  direction: SortDirection
  onChange: (field: string, direction: SortDirection) => void
}

const ListSortMenu = ({ options, field, direction, onChange }: ListSortMenuProps) => {
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
        <DropdownMenuRadioGroup value={field} onValueChange={(newField) => onChange(newField, direction)}>
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
