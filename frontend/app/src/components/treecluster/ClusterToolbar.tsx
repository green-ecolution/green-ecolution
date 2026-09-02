import React, { useEffect, useState } from 'react'
import { useNavigate, useSearch } from '@tanstack/react-router'
import { ArrowDownUp, Search } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { Combobox, Input } from '@green-ecolution/ui'
import type { ComboboxOption } from '@green-ecolution/ui'

const ClusterToolbar: React.FC = () => {
  const { t } = useTranslation('treecluster')
  const search = useSearch({ strict: false })
  const navigate = useNavigate()
  const sortOptions: ComboboxOption[] = [
    { value: 'name', label: t('toolbar.sortOptionName') },
    { value: 'moisture', label: t('toolbar.sortOptionMoisture') },
    { value: 'trees', label: t('toolbar.sortOptionTrees') },
  ]

  const [inputValue, setInputValue] = useState(search.q ?? '')

  useEffect(() => {
    const timeout = setTimeout(() => {
      const term = inputValue.trim()
      navigate({
        to: '/treecluster',
        search: (prev) => ({
          ...prev,
          q: term || undefined,
          page: 1,
        }),
      }).catch((error) => console.error('Navigation failed:', error))
    }, 300)

    return () => clearTimeout(timeout)
  }, [inputValue, navigate])

  const handleSortChange = (value: string) => {
    navigate({
      to: '/treecluster',
      search: (prev) => ({
        ...prev,
        sort: value as ('name' | 'moisture' | 'trees') | undefined,
        page: 1,
      }),
    }).catch((error) => console.error('Navigation failed:', error))
  }

  return (
    <div className="flex flex-1 items-center gap-3 sm:flex-initial sm:flex-wrap">
      <div className="relative flex-1 sm:max-w-xs sm:flex-1">
        <Search
          className="pointer-events-none absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-dark-400"
          aria-hidden
        />
        <Input
          type="search"
          placeholder={t('toolbar.searchPlaceholder')}
          value={inputValue}
          onChange={(e) => setInputValue(e.target.value)}
          className="pl-9"
        />
      </div>

      <div className="hidden items-center gap-2 sm:flex">
        <ArrowDownUp className="h-4 w-4 shrink-0 text-dark-600" aria-hidden />
        <span className="shrink-0 whitespace-nowrap text-sm text-dark-600">
          {t('toolbar.sortLabel')}
        </span>
        <Combobox
          options={sortOptions}
          value={search.sort ?? 'name'}
          onChange={handleSortChange}
          placeholder={t('toolbar.sortPlaceholder')}
          searchPlaceholder={t('toolbar.sortSearchPlaceholder')}
          aria-label={t('toolbar.sortAriaLabel')}
          className="min-w-0 flex-1 sm:w-48 sm:flex-none"
        />
      </div>
    </div>
  )
}

export default ClusterToolbar
