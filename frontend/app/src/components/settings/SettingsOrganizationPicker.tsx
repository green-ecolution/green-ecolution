import { useState } from 'react'
import { Link } from '@tanstack/react-router'
import { Check, ChevronRight, ChevronsUpDown } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import {
  Command,
  CommandEmpty,
  CommandInput,
  CommandItem,
  CommandList,
  Popover,
  PopoverContent,
  PopoverTrigger,
} from '@green-ecolution/ui'
import type { OrganizationResponse } from '@/api/backendApi'
import { buildOrganizationRows } from '@/lib/auth/scope/organizationRows'
import { initialsOfName } from '@/lib/initials'
import { SECTION_ROUTES } from './settingsSections'

interface SettingsOrganizationPickerProps {
  organizations: OrganizationResponse[]
  selectedId: string
  onSelect: (orgId: string) => void
}

const TILE =
  'flex size-8 shrink-0 items-center justify-center rounded-lg bg-green-dark text-sm font-semibold text-white'

const SettingsOrganizationPicker = ({
  organizations,
  selectedId,
  onSelect,
}: SettingsOrganizationPickerProps) => {
  const { t } = useTranslation('settings')
  const [open, setOpen] = useState(false)
  const [query, setQuery] = useState('')

  const selected = organizations.find((org) => org.id === selectedId)
  if (!selected) return null
  const { name } = selected
  const childCount = organizations.filter((org) => org.parentId === selectedId).length

  const summary = (
    <>
      <span className={TILE} aria-hidden>
        {initialsOfName(name)}
      </span>
      <span className="min-w-0 flex-1 text-left">
        <span className="block truncate font-lato text-[15px] font-bold text-dark">{name}</span>
        {childCount > 0 && (
          <span className="block truncate text-xs text-dark-600">
            {t('picker.withChildren', { count: childCount })}
          </span>
        )}
      </span>
    </>
  )

  if (organizations.length <= 1) {
    return <div className="flex min-h-14 items-center gap-3 px-3">{summary}</div>
  }

  const rows = buildOrganizationRows(organizations, query)

  return (
    <Popover open={open} onOpenChange={setOpen}>
      <PopoverTrigger asChild>
        <button
          type="button"
          aria-label={t('picker.ariaLabel', { name })}
          className="flex min-h-14 w-full items-center gap-3 rounded-xl border border-dark-200 bg-white px-3 shadow-cards focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
        >
          {summary}
          <ChevronsUpDown className="size-4 shrink-0 text-dark-600" aria-hidden />
        </button>
      </PopoverTrigger>
      <PopoverContent align="start" className="w-[22rem] max-w-[calc(100vw-2rem)] p-2">
        <Command shouldFilter={false}>
          <CommandInput
            value={query}
            onValueChange={setQuery}
            placeholder={t('picker.searchPlaceholder')}
          />
          <CommandList>
            <CommandEmpty>{t('picker.empty')}</CommandEmpty>
            {rows.map((row) => (
              <CommandItem
                key={row.id}
                value={row.id}
                onSelect={() => {
                  setOpen(false)
                  setQuery('')
                  if (row.id !== selectedId) onSelect(row.id)
                }}
                style={{ paddingLeft: `${0.625 + row.depth * 1.25}rem` }}
              >
                <span className="min-w-0 flex-1 truncate">{row.name}</span>
                {row.id === selectedId && <Check className="size-4 shrink-0" aria-hidden />}
              </CommandItem>
            ))}
          </CommandList>
        </Command>
        <Link
          to={SECTION_ROUTES.structure}
          params={{ orgId: selectedId }}
          onClick={() => setOpen(false)}
          className="mt-2 flex min-h-11 items-center justify-between gap-3 border-t border-dark-100 px-2.5 text-sm font-bold text-green-dark"
        >
          {t('picker.manageStructure')}
          <ChevronRight className="size-4" aria-hidden />
        </Link>
      </PopoverContent>
    </Popover>
  )
}

export default SettingsOrganizationPicker
