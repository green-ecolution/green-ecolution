import { useCallback } from 'react'
import { Link, useNavigate, useSearch } from '@tanstack/react-router'
import { Plus } from 'lucide-react'
import { WateringStatus } from '@green-ecolution/backend-client'
import { Button } from '@green-ecolution/ui'
import useStore from '@/store/store'
import Dialog from '@/components/general/filter/Dialog'
import StatusFieldset from '@/components/general/filter/fieldsets/StatusFieldset'
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
          <div className="flex items-center gap-2 lg:ml-auto">
            <MapButtons />
            <Button asChild>
              <Link to="/map/treecluster/new" search={(prev) => prev}>
                <span className="hidden sm:inline">Gruppe anlegen</span>
                <Plus />
              </Link>
            </Button>
          </div>
        }
      />
    </div>
  )
}

export default MapToolbarBar
