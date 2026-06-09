import { createFileRoute, useNavigate } from '@tanstack/react-router'
import MapButtons from '@/components/map/MapButtons'
import type { ClusterMarkerResponse, Tree, TreeCluster, TreeMarkerResponse } from '@/api/backendApi'
import { useCallback, useMemo, useRef, useState } from 'react'
import Dialog from '@/components/general/filter/Dialog'
import StatusFieldset from '@/components/general/filter/fieldsets/StatusFieldset'
import FilterProvider from '@/context/FilterContext'
import { z } from 'zod'
import ClusterFieldset from '@/components/general/filter/fieldsets/ClusterFieldset'
import PlantingYearFieldset from '@/components/general/filter/fieldsets/PlantingYearFieldset'
import useMapInteractions from '@/hooks/useMapInteractions'
import { WithTreesAndClusters } from '@/components/map/marker/WithAllClusterAndTrees'
import WithFilterdTrees from '@/components/map/marker/WithFilterdTrees'
import { filterSearchSchema } from '@/lib/filterSearchSchema'
import { WateringStatus } from '@green-ecolution/backend-client'
import { Plus } from 'lucide-react'
import ButtonLink from '@/components/general/links/ButtonLink'
import MapFilterToolbar from '@/components/map/MapFilterToolbar'
import ClusterPanel from '@/components/map/cluster-panel/ClusterPanel'

const mapFilterSchema = filterSearchSchema
  .pick({ wateringStatuses: true, hasCluster: true, plantingYears: true })
  .extend({
    tree: z.string().optional(),
    cluster: z.string().optional(),
  })

function MapView() {
  const navigate = useNavigate({ from: '/map' })
  const search = Route.useSearch()
  const { enableDragging, disableDragging } = useMapInteractions()
  const dialogRef = useRef<HTMLDivElement>(null)
  const [searchTerm, setSearchTerm] = useState('')

  const hasActiveFilter = useMemo(
    () =>
      search.wateringStatuses !== undefined ||
      search.hasCluster !== undefined ||
      search.plantingYears !== undefined,
    [search.wateringStatuses, search.hasCluster, search.plantingYears],
  )

  const handleTreeClick = useCallback(
    (tree: TreeMarkerResponse | Tree) => {
      navigate({ to: `/trees/$treeId`, params: { treeId: tree.id.toString() } }).catch((error) =>
        console.error('Navigation failed:', error),
      )
    },
    [navigate],
  )

  const handleClusterClick = useCallback(
    (cluster: ClusterMarkerResponse | TreeCluster) => {
      navigate({
        to: '/map',
        search: (prev) => ({ ...prev, cluster: cluster.id.toString() }),
      }).catch((error) => console.error('Navigation failed:', error))
    },
    [navigate],
  )

  const handleClosePanel = useCallback(() => {
    navigate({ to: '/map', search: (prev) => ({ ...prev, cluster: undefined }) }).catch((error) =>
      console.error('Navigation failed:', error),
    )
  }, [navigate])

  const handleOpenDashboard = useCallback(() => {
    if (!search.cluster) return
    navigate({
      to: '/treecluster/$treeclusterId',
      params: { treeclusterId: search.cluster },
    }).catch((error) => console.error('Navigation failed:', error))
  }, [navigate, search.cluster])

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

  const handleMapInteractions = useCallback(
    (isOpen: boolean) => {
      if (isOpen) {
        disableDragging()
      } else {
        enableDragging()
      }
    },
    [disableDragging, enableDragging],
  )

  return (
    <>
      <MapFilterToolbar
        searchTerm={searchTerm}
        onSearchTermChange={setSearchTerm}
        statuses={search.wateringStatuses ?? []}
        onToggleStatus={handleToggleStatus}
        filterSlot={
          <Dialog
            ref={dialogRef}
            headline="Bäume filtern"
            isOnMap
            fullUrlPath={Route.fullPath}
            onToggleOpen={handleMapInteractions}
          >
            <StatusFieldset />
            <ClusterFieldset />
            <PlantingYearFieldset />
          </Dialog>
        }
        createSlot={<ButtonLink icon={Plus} label="Gruppe anlegen" link={{ to: '/treecluster/new' }} />}
      />
      <MapButtons />
      {hasActiveFilter ? (
        <WithFilterdTrees
          onClick={handleTreeClick}
          selectedTrees={search.tree ? [search.tree] : []}
          hasHighlightedTree={search.tree}
          hasCluster={search.hasCluster}
          plantingYears={search.plantingYears}
          wateringStatuses={search.wateringStatuses}
        />
      ) : (
        <WithTreesAndClusters
          onClickTree={handleTreeClick}
          onClickCluster={handleClusterClick}
          hasHighlightedTree={search.tree}
          hasHighlightedCluster={search.cluster}
          nameFilter={searchTerm}
        />
      )}
      {search.cluster && (
        <ClusterPanel
          clusterId={search.cluster}
          onClose={handleClosePanel}
          onOpenDashboard={handleOpenDashboard}
        />
      )}
    </>
  )
}

const MapViewWithProvider = () => (
  <FilterProvider>
    <MapView />
  </FilterProvider>
)

export const Route = createFileRoute('/_protected/map/')({
  component: MapViewWithProvider,
  validateSearch: mapFilterSchema,
})
