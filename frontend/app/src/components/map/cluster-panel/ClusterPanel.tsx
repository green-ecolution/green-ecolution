import { useQuery } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'
import { Button, Loading } from '@green-ecolution/ui'
import { Pencil } from 'lucide-react'
import { clusterQueries, isValidUuid } from '@/api/queries'
import MapPanel from '@/components/map-gl/MapPanel'
import { useHasPermission } from '@/lib/auth/useHasPermission'
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
  const { t } = useTranslation('map')
  const { data, isError } = useQuery(clusterQueries.detail(clusterId))
  const failed = !isValidUuid(clusterId) || isError
  const canEdit = useHasPermission(['tree_cluster:update'])

  const headerAction =
    data && canEdit ? (
      <Button variant="ghost" size="icon" aria-label={t('cluster.editAriaLabel')} onClick={onEdit}>
        <Pencil />
      </Button>
    ) : undefined

  return (
    <MapPanel
      title={data?.name ?? t('cluster.fallbackTitle')}
      headerAction={headerAction}
      onClose={onClose}
      closeLabel={t('cluster.closePanelAriaLabel')}
      mobileCollapsedSnap="260px"
      activeSnapPoint={activeSnapPoint}
      setActiveSnapPoint={setActiveSnapPoint}
    >
      <ClusterPanelShell onClose={onClose}>
        {data ? (
          <ClusterPanelView treecluster={data} onOpenDashboard={onOpenDashboard} />
        ) : failed ? (
          <p className="py-10 text-center text-dark-600">{t('cluster.loadError')}</p>
        ) : (
          <Loading className="justify-center py-10" label={t('cluster.loadingLabel')} />
        )}
      </ClusterPanelShell>
    </MapPanel>
  )
}

export default ClusterPanel
