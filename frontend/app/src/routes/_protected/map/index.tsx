import { createFileRoute, useNavigate } from '@tanstack/react-router'
import MapButtons from '@/components/map/MapButtons'
import type { ClusterMarkerResponse, Tree, TreeCluster, TreeMarkerResponse } from '@/api/backendApi'
import { useCallback, useMemo, useRef } from 'react'
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

  const handleBoundaryClick = useCallback(
    (clusterId: string) => {
      navigate({ to: '/map', search: (prev) => ({ ...prev, cluster: clusterId }) }).catch((error) =>
        console.error('Navigation failed:', error),
      )
    },
    [navigate],
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
      <div className="absolute top-6 left-4 z-[1000]">
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
      </div>
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
          onClickBoundary={handleBoundaryClick}
          hasHighlightedTree={search.tree}
          hasHighlightedCluster={search.cluster}
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
