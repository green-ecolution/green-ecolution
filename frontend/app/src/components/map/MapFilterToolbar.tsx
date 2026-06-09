import { ReactNode, useState } from 'react'
import { Search, SlidersHorizontal } from 'lucide-react'
import { WateringStatus } from '@green-ecolution/backend-client'
import { Input } from '@green-ecolution/ui'
import { getWateringStatusDetails } from '@/hooks/details/useDetailsForWateringStatus'

interface MapFilterToolbarProps {
  searchTerm: string
  onSearchTermChange: (value: string) => void
  statuses: WateringStatus[]
  onToggleStatus: (status: WateringStatus) => void
  filterSlot: ReactNode
  createSlot: ReactNode
}

const QUICK_STATUSES: WateringStatus[] = [WateringStatus.Bad, WateringStatus.Moderate]

const MapFilterToolbar = ({
  searchTerm,
  onSearchTermChange,
  statuses,
  onToggleStatus,
  filterSlot,
  createSlot,
}: MapFilterToolbarProps) => {
  const [expanded, setExpanded] = useState(false)
  const highlightToggle = expanded || statuses.length > 0

  return (
    <div className="flex w-full flex-wrap items-center gap-2 font-nunito-sans">
      <div className="relative flex-1 lg:w-56 lg:flex-none">
        <Search className="pointer-events-none absolute left-3 top-1/2 size-4 -translate-y-1/2 text-dark-400" />
        <Input
          value={searchTerm}
          onChange={(event) => onSearchTermChange(event.target.value)}
          placeholder="Baumgruppe…"
          className="w-full rounded-full border-dark-200 bg-white pl-9 shadow-cards focus-visible:border-green-dark"
        />
      </div>

      <button
        type="button"
        aria-label="Filter und Aktionen"
        aria-expanded={expanded}
        onClick={() => setExpanded((value) => !value)}
        className={`flex size-10 shrink-0 cursor-pointer items-center justify-center rounded-full border shadow-cards transition-colors lg:hidden ${
          highlightToggle
            ? 'border-green-dark bg-green-dark-50 text-green-dark'
            : 'border-dark-200 bg-white text-dark-700'
        }`}
      >
        <SlidersHorizontal className="size-5" />
      </button>

      <div
        className={`${expanded ? 'flex' : 'hidden'} w-full flex-wrap items-center gap-2 lg:flex lg:w-auto lg:flex-1`}
      >
        {filterSlot}

        <div className="flex gap-2">
          {QUICK_STATUSES.map((status) => {
            const details = getWateringStatusDetails(status)
            const active = statuses.includes(status)
            return (
              <button
                key={status}
                type="button"
                aria-pressed={active}
                onClick={() => onToggleStatus(status)}
                className={`flex h-10 cursor-pointer items-center gap-2 rounded-full border px-4 text-sm shadow-cards transition-colors ${
                  active
                    ? 'border-dark-300 bg-dark-100 text-dark-900'
                    : 'border-dark-200 bg-white text-dark-700 hover:bg-dark-50'
                }`}
              >
                <span
                  className="size-2 rounded-full"
                  style={{ backgroundColor: details.colorHex }}
                  aria-hidden="true"
                />
                {details.label}
              </button>
            )
          })}
        </div>

        {createSlot}
      </div>
    </div>
  )
}

export default MapFilterToolbar
