import { useQuery } from '@tanstack/react-query'
import { Button, Loading } from '@green-ecolution/ui'
import { Pencil } from 'lucide-react'
import { isValidUuid, treeClusterIdQuery } from '@/api/queries'
import MapPanel from '@/components/map-gl/MapPanel'
import ClusterPanelShell from './ClusterPanelShell'
import ClusterPanelView from './ClusterPanelView'

interface ClusterPanelProps {
  clusterId: string
  onClose: () => void
  onOpenDashboard: () => void
  onEdit: () => void
  activeSnapPoint?: number | string | null
  setActiveSnapPoint?: (snap: number | string | null) => void
}

const ClusterPanel = ({
  clusterId,
  onClose,
  onOpenDashboard,
  onEdit,
  activeSnapPoint,
  setActiveSnapPoint,
}: ClusterPanelProps) => {
  const { data, isError } = useQuery(treeClusterIdQuery(clusterId))
  const failed = !isValidUuid(clusterId) || isError

  const headerAction = data ? (
    <Button variant="ghost" size="icon" aria-label="Gruppe bearbeiten" onClick={onEdit}>
      <Pencil />
    </Button>
  ) : undefined

  return (
    <MapPanel
      title={data?.name ?? 'Baumgruppe'}
      headerAction={headerAction}
      onClose={onClose}
      closeLabel="Seitenansicht schließen"
      mobileCollapsedSnap="260px"
      activeSnapPoint={activeSnapPoint}
      setActiveSnapPoint={setActiveSnapPoint}
    >
      <ClusterPanelShell onClose={onClose}>
        {data ? (
          <ClusterPanelView treecluster={data} onOpenDashboard={onOpenDashboard} />
        ) : failed ? (
          <p className="py-10 text-center text-dark-600">
            Die Baumgruppe konnte nicht geladen werden.
          </p>
        ) : (
          <Loading className="justify-center py-10" label="Lade Baumgruppe..." />
        )}
      </ClusterPanelShell>
    </MapPanel>
  )
}

export default ClusterPanel
