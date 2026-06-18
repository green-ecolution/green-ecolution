import React, { useEffect, useState } from 'react'
import { useNavigate, useSearch } from '@tanstack/react-router'
import { ArrowDownUp, Search } from 'lucide-react'
import { Combobox, Input } from '@green-ecolution/ui'
import type { ComboboxOption } from '@green-ecolution/ui'

const SORT_OPTIONS: ComboboxOption[] = [
  { value: 'name', label: 'Name' },
  { value: 'moisture', label: 'Feuchte' },
  { value: 'trees', label: 'Anzahl Bäume' },
]

const ClusterToolbar: React.FC = () => {
  const search = useSearch({ strict: false })
  const navigate = useNavigate()

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
    <div className="flex w-full flex-col gap-3 sm:w-auto sm:flex-row sm:flex-wrap sm:items-center">
      <div className="relative w-full sm:max-w-xs sm:flex-1">
        <Search
          className="pointer-events-none absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-dark-400"
          aria-hidden
        />
        <Input
          type="search"
          placeholder="Gruppe suchen"
          value={inputValue}
          onChange={(e) => setInputValue(e.target.value)}
          className="pl-9"
        />
      </div>

      <div className="flex w-full items-center gap-2 sm:w-auto">
        <ArrowDownUp className="h-4 w-4 shrink-0 text-dark-600" aria-hidden />
        <Combobox
          options={SORT_OPTIONS}
          value={search.sort ?? 'name'}
          onChange={handleSortChange}
          placeholder="Sortieren"
          searchPlaceholder="Sortierung suchen"
          className="w-full sm:w-44"
        />
      </div>
    </div>
  )
}

export default ClusterToolbar
