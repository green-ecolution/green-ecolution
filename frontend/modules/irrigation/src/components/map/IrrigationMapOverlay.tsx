import { useCallback, useState } from 'react'
import { useLocation, useNavigate } from '@tanstack/react-router'
import { useIrrigationMapSearch } from '../../lib/mapSearch'
import ClusterPanel from './cluster-panel/ClusterPanel'

const COLLAPSED_SNAP = '260px'

const IrrigationMapOverlay = () => {
  const { pathname } = useLocation()
  const navigate = useNavigate()
  const { cluster } = useIrrigationMapSearch()
  // Reactive exact-match: matchRoute() didn't reliably re-render on
  // pathname-only changes, leaving isIndex stale across /map ↔ sub-route nav.
  const isIndex = pathname === '/map' || pathname === '/map/'
  const panelClusterId = isIndex ? cluster : undefined

  const [snapPoint, setSnapPoint] = useState<number | string | null>(COLLAPSED_SNAP)
  // A background click on the map closes the panel by dropping `cluster` from
  // the URL; the next panel has to open collapsed again, as after the close button.
  const [openedCluster, setOpenedCluster] = useState(cluster)
  if (cluster !== openedCluster) {
    setOpenedCluster(cluster)
    if (!cluster) setSnapPoint(COLLAPSED_SNAP)
  }

  const handleClosePanel = useCallback(() => {
    setSnapPoint(COLLAPSED_SNAP)
    navigate({ to: '/map', search: (prev) => ({ ...prev, cluster: undefined }) }).catch((error) =>
      console.error('Navigation failed:', error),
    )
  }, [navigate])

  const handleEditCluster = useCallback(() => {
    if (!panelClusterId) return
    navigate({
      to: '/map/treecluster/edit/$treeclusterId',
      params: { treeclusterId: panelClusterId },
      search: (prev) => prev,
    }).catch((error) => console.error('Navigation failed:', error))
  }, [navigate, panelClusterId])

  const handleOpenDashboard = useCallback(() => {
    if (!panelClusterId) return
    navigate({
      to: '/treecluster/$treeclusterId',
      params: { treeclusterId: panelClusterId },
    }).catch((error) => console.error('Navigation failed:', error))
  }, [navigate, panelClusterId])

  if (!panelClusterId) return null

  return (
    <ClusterPanel
      key={panelClusterId}
      clusterId={panelClusterId}
      onClose={handleClosePanel}
      onOpenDashboard={handleOpenDashboard}
      onEdit={handleEditCluster}
      activeSnapPoint={snapPoint}
      setActiveSnapPoint={setSnapPoint}
    />
  )
}

export default IrrigationMapOverlay
