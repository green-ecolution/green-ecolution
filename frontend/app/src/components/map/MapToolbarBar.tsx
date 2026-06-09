import { useCallback } from 'react'
import { useNavigate, useSearch } from '@tanstack/react-router'
import { Plus } from 'lucide-react'
import { WateringStatus } from '@green-ecolution/backend-client'
import useStore from '@/store/store'
import FilterProvider from '@/context/FilterContext'
import Dialog from '@/components/general/filter/Dialog'
import StatusFieldset from '@/components/general/filter/fieldsets/StatusFieldset'
import ButtonLink from '@/components/general/links/ButtonLink'
import MapFilterToolbar from './MapFilterToolbar'
import MapButtons from './MapButtons'

const MapToolbarBar = () => {
  const navigate = useNavigate()
  const search = useSearch({ strict: false })
  const searchTerm = useStore((state) => state.mapSearchTerm)
  const setSearchTerm = useStore((state) => state.setMapSearchTerm)

  const handleToggleStatus = useCallback(
    (status: WateringStatus) => {
      const current = search.wateringStatuses ?? []
      const next = current.includes(status)
        ? current.filter((s) => s !== status)
        : [...current, status]
      navigate({
        to: '/map',
        search: (prev) => ({ ...prev, wateringStatuses: next.length ? next : undefined }),
      }).catch((error) => console.error('Navigation failed:', error))
    },
    [navigate, search.wateringStatuses],
  )

  return (
    <div className="flex shrink-0 items-center gap-2 border-b border-dark-100 bg-[#FCFCFC] px-4 py-3">
      <FilterProvider>
        <MapFilterToolbar
          searchTerm={searchTerm}
          onSearchTermChange={setSearchTerm}
          statuses={search.wateringStatuses ?? []}
          onToggleStatus={handleToggleStatus}
          filterSlot={
            <Dialog headline="Baumgruppen filtern" isOnMap fullUrlPath="/map">
              <StatusFieldset />
            </Dialog>
          }
          createSlot={
            <div className="ml-auto flex items-center gap-2">
              <MapButtons />
              <ButtonLink icon={Plus} label="Gruppe anlegen" link={{ to: '/treecluster/new' }} />
            </div>
          }
        />
      </FilterProvider>
    </div>
  )
}

export default MapToolbarBar
