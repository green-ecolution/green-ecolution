import { useQuery } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'
import { WateringStatus } from '@green-ecolution/backend-client'
import DashboardStatTile from '@/components/dashboard/DashboardStatTile'
import { useHasPermission } from '@/lib/auth/useHasPermission'
import { clusterQueries } from '../../api/queries'

const ClustersTile = () => {
  const { t } = useTranslation('irrigation')
  const canRead = useHasPermission(['tree_cluster:read'])
  const { data: stats } = useQuery({ ...clusterQueries.statistics(), enabled: canRead })
  if (!canRead) return null

  return (
    <DashboardStatTile
      label={t('dashboard.clusters.label')}
      value={stats?.bad}
      status={stats?.bad ? 'red' : stats?.moderate ? 'yellow' : 'green-dark'}
      description={
        stats
          ? t('dashboard.clusters.description', { moderate: stats.moderate, total: stats.total })
          : undefined
      }
      link={{
        to: '/treecluster',
        search: { page: 1, wateringStatuses: [WateringStatus.Bad] },
      }}
    />
  )
}

export default ClustersTile
