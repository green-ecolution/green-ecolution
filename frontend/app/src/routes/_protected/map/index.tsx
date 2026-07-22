import { createFileRoute, useNavigate } from '@tanstack/react-router'
import { useCallback } from 'react'
import { z } from 'zod'
import { filterSearchSchema } from '@/lib/filterSearchSchema'
import useClusterBoundaryLayer from '@/components/map-gl/layers/useClusterBoundaryLayer'
import useClusterMarkerLayer from '@/components/map-gl/layers/useClusterMarkerLayer'
import useTreeLayers from '@/components/map-gl/layers/useTreeLayers'
import MapStatusLegend from '@/components/map/MapStatusLegend'

const mapFilterSchema = filterSearchSchema.pick({ wateringStatuses: true }).extend({
  tree: z.string().optional(),
  cluster: z.string().optional(),
  q: z.string().optional().catch(undefined),
})

function MapView() {
  const navigate = useNavigate({ from: '/map' })
  const search = Route.useSearch()
  const searchTerm = search.q ?? ''

  const handleTreeClick = useCallback(
    (treeId: string) => {
      navigate({ to: '/trees/$treeId', params: { treeId } }).catch((error) =>
        console.error('Navigation failed:', error),
      )
    },
    [navigate],
  )

  const handleOpenCluster = useCallback(
    (clusterId: string) => {
      navigate({ to: '/map', search: (prev) => ({ ...prev, cluster: clusterId }) }).catch((error) =>
        console.error('Navigation failed:', error),
      )
    },
    [navigate],
  )

  useClusterBoundaryLayer({
    onBoundaryClick: handleOpenCluster,
    selectedClusterId: search.cluster,
    wateringStatuses: search.wateringStatuses,
    nameFilter: searchTerm,
  })
  useClusterMarkerLayer({
    onClusterClick: handleOpenCluster,
    wateringStatuses: search.wateringStatuses,
    nameFilter: searchTerm,
  })
  useTreeLayers({ onTreeClick: handleTreeClick, wateringStatuses: search.wateringStatuses })

  return <MapStatusLegend />
}

export const Route = createFileRoute('/_protected/map/')({
  component: MapView,
  validateSearch: mapFilterSchema,
})
