import React from 'react'
import { useNavigate, useSearch } from '@tanstack/react-router'
import { LayoutGrid, Table } from 'lucide-react'

type ViewOption = 'cards' | 'table'

const OPTIONS: { value: ViewOption; label: string; icon: React.ElementType }[] = [
  { value: 'cards', label: 'Karten', icon: LayoutGrid },
  { value: 'table', label: 'Tabelle', icon: Table },
]

const ClusterViewToggle: React.FC = () => {
  const search = useSearch({ strict: false })
  const view: ViewOption =
    'view' in search && (search.view === 'cards' || search.view === 'table') ? search.view : 'cards'
  const navigate = useNavigate()
  const page = typeof search.page === 'number' ? search.page : 1

  const handleSelect = (v: ViewOption) => {
    navigate({
      to: '/treecluster',
      search: (prev: Record<string, unknown>) => ({ ...prev, page, view: v }),
    }).catch((error) => console.error('Navigation failed:', error))
  }

  return (
    <div
      role="tablist"
      aria-label="Ansicht wechseln"
      className="inline-flex rounded-lg bg-dark-50 p-1"
    >
      {OPTIONS.map(({ value, label, icon: Icon }) => (
        <button
          key={value}
          type="button"
          role="tab"
          aria-selected={view === value}
          onClick={() => handleSelect(value)}
          className={[
            'flex items-center gap-1.5 rounded-md px-3 py-1.5 text-sm transition',
            view === value
              ? 'bg-white text-green-dark shadow-sm'
              : 'text-dark-600 hover:text-dark-900',
          ].join(' ')}
        >
          <Icon className="h-4 w-4" aria-hidden />
          {label}
        </button>
      ))}
    </div>
  )
}

export default ClusterViewToggle
