import { useCallback } from 'react'
import { useNavigate } from '@tanstack/react-router'
import MapBackgroundClick from '@/components/map-gl/MapBackgroundClick'
import useClusterBoundaryLayer from '../map-gl/layers/useClusterBoundaryLayer'
import useClusterMarkerLayer from '../map-gl/layers/useClusterMarkerLayer'
import useTreeLayers from '../map-gl/layers/useTreeLayers'
import { useIrrigationMapSearch } from '../../lib/mapSearch'
import MapStatusLegend from './MapStatusLegend'

const IrrigationMapOverview = () => {
  const navigate = useNavigate({ from: '/map' })
  const search = useIrrigationMapSearch()
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

  const handleClosePanel = useCallback(() => {
    navigate({ to: '/map', search: (prev) => ({ ...prev, cluster: undefined }) }).catch((error) =>
      console.error('Navigation failed:', error),
    )
  }, [navigate])

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

  return (
    <>
      <MapBackgroundClick onBackground={handleClosePanel} />
      <MapStatusLegend />
    </>
  )
}

export default IrrigationMapOverview
