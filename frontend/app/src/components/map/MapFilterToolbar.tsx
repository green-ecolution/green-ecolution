import { ReactNode } from 'react'
import { Search } from 'lucide-react'
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
  /** Namenssuche greift nur in der Gruppen-Ansicht; bei aktivem Filter zeigt die Karte Bäume. */
  searchDisabled?: boolean
}

const QUICK_STATUSES: WateringStatus[] = [WateringStatus.Bad, WateringStatus.Moderate]

const MapFilterToolbar = ({
  searchTerm,
  onSearchTermChange,
  statuses,
  onToggleStatus,
  filterSlot,
  createSlot,
  searchDisabled = false,
}: MapFilterToolbarProps) => (
  <div className="absolute left-4 right-4 top-4 z-[1000] flex flex-wrap items-center gap-2 font-nunito-sans lg:right-auto">
    <div className="relative">
      <Search className="pointer-events-none absolute left-3 top-1/2 size-4 -translate-y-1/2 text-dark-400" />
      <Input
        value={searchTerm}
        onChange={(event) => onSearchTermChange(event.target.value)}
        placeholder="Baumgruppe…"
        disabled={searchDisabled}
        title={searchDisabled ? 'Suche ist bei aktivem Filter nicht verfügbar' : undefined}
        className="w-56 rounded-full border-transparent bg-white pl-9 shadow-cards focus-visible:border-green-dark disabled:opacity-60"
      />
    </div>

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
            className={`flex items-center gap-2 rounded-full border px-3 py-2 text-sm shadow-cards transition-colors ${
              active
                ? 'border-dark-300 bg-dark-100 text-dark-900'
                : 'border-transparent bg-white text-dark-700 hover:bg-dark-50'
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

    <div className="ml-auto lg:ml-2">{createSlot}</div>
  </div>
)

export default MapFilterToolbar
