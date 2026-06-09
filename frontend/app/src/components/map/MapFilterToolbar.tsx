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
}

const QUICK_STATUSES: WateringStatus[] = [WateringStatus.Bad, WateringStatus.Moderate]

const MapFilterToolbar = ({
  searchTerm,
  onSearchTermChange,
  statuses,
  onToggleStatus,
  filterSlot,
  createSlot,
}: MapFilterToolbarProps) => (
  <div className="absolute top-4 left-4 right-4 z-[1000] flex flex-wrap items-center gap-3 font-nunito-sans lg:right-auto">
    <div className="relative">
      <Search className="absolute left-3 top-1/2 size-4 -translate-y-1/2 text-dark-400" />
      <Input
        value={searchTerm}
        onChange={(event) => onSearchTermChange(event.target.value)}
        placeholder="Baumgruppe…"
        className="bg-white pl-9 shadow-cards"
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
            className={`rounded-full border px-3 py-1.5 text-sm shadow-cards transition-colors ${
              active ? 'bg-dark-100 border-dark-300' : 'bg-white border-dark-200'
            }`}
          >
            {details.label}
          </button>
        )
      })}
    </div>

    <div className="ml-auto lg:ml-4">{createSlot}</div>
  </div>
)

export default MapFilterToolbar
