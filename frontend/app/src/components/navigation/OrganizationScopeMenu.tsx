import { useState } from 'react'
import { Building2, Check, Search } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import {
  DropdownMenuRadioGroup,
  DropdownMenuRadioItem,
  DropdownMenuSub,
  DropdownMenuSubContent,
  DropdownMenuSubTrigger,
} from '@green-ecolution/ui'
import { buildOrganizationRows } from '@/lib/auth/scope/organizationRows'
import { useScopeOrganizations } from '@/lib/auth/scope/useScopeOrganizations'
import { useSwitchOrganizationScope } from '@/lib/auth/scope/useSwitchOrganizationScope'
import { menuItemClasses } from './menuItemStyles'

const SEARCH_THRESHOLD = 8

const radioItemClasses = `${menuItemClasses} relative justify-between pr-8 [&>span:first-child]:hidden`

const OrganizationScopeMenu = () => {
  const { t } = useTranslation('navigation')
  const { organizations, switchable, homeId, activeId, activeName } = useScopeOrganizations()
  const switchScope = useSwitchOrganizationScope()
  const [query, setQuery] = useState('')

  if (!switchable) return null

  const rows = buildOrganizationRows(organizations, query)

  return (
    <DropdownMenuSub>
      <DropdownMenuSubTrigger className={`${menuItemClasses} data-[state=open]:bg-green-light/20`}>
        <Building2 />
        {t('user.organization')}
        <span className="ml-auto max-w-32 flex-1 truncate text-right text-xs font-normal text-light/70">
          {activeName}
        </span>
      </DropdownMenuSubTrigger>
      <DropdownMenuSubContent className="max-h-80 w-72 overflow-y-auto rounded-lg border-dark-400/30 bg-dark p-2 text-sm text-white shadow-cards">
        {organizations.length > SEARCH_THRESHOLD && (
          <div className="relative px-1 pb-2">
            <Search
              className="pointer-events-none absolute left-4 top-2.5 size-4 text-light/70"
              aria-hidden
            />
            <input
              type="search"
              value={query}
              onChange={(event) => setQuery(event.target.value)}
              onKeyDown={(event) => {
                // Keeps the menu's typeahead from swallowing typed characters.
                event.stopPropagation()
                // The menu's own arrow navigation only starts from an item.
                if (event.key === 'ArrowDown') {
                  event.preventDefault()
                  event.currentTarget
                    .closest('[role="menu"]')
                    ?.querySelector<HTMLElement>('[role="menuitemradio"]')
                    ?.focus()
                }
              }}
              placeholder={t('user.searchOrganization')}
              aria-label={t('user.searchOrganization')}
              className="w-full rounded-lg border border-dark-400/40 bg-dark-800 py-2 pl-9 pr-3 text-light placeholder:text-light/50 focus:outline-none focus-visible:ring-2 focus-visible:ring-green-light"
            />
          </div>
        )}
        <DropdownMenuRadioGroup
          value={activeId ?? ''}
          // Back in the home organization means no scope: the unnarrowed view.
          onValueChange={(value) => void switchScope(value === homeId ? null : value)}
        >
          {rows.map((row) => (
            <DropdownMenuRadioItem
              key={row.id}
              value={row.id}
              className={radioItemClasses}
              style={{ paddingLeft: `${0.75 + row.depth}rem` }}
            >
              <span className="truncate">{row.name}</span>
              {activeId === row.id && <Check className="absolute right-3" aria-hidden />}
            </DropdownMenuRadioItem>
          ))}
        </DropdownMenuRadioGroup>
      </DropdownMenuSubContent>
    </DropdownMenuSub>
  )
}

export default OrganizationScopeMenu
