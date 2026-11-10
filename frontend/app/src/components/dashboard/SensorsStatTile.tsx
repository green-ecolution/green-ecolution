import { useQuery } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'
import { SensorStatus } from '@green-ecolution/backend-client'
import { sensorQueries } from '@/api/queries'
import { useHasPermission } from '@/lib/auth/useHasPermission'
import DashboardStatTile from './DashboardStatTile'

const countOf = (statuses: SensorStatus[]) =>
  sensorQueries.list({ page: 1, perPage: 1, status: statuses })

const SensorsStatTile = () => {
  const { t } = useTranslation('dashboard')
  const canRead = useHasPermission(['sensor:read'])
  const offline = useQuery({ ...countOf([SensorStatus.Offline]), enabled: canRead })
  const activated = useQuery({
    ...countOf([SensorStatus.Online, SensorStatus.Offline]),
    enabled: canRead,
  })
  if (!canRead) return null

  const offlineCount = offline.data?.pagination?.totalRecords
  const activatedCount = activated.data?.pagination?.totalRecords

  return (
    <DashboardStatTile
      label={t('overview.sensors.label')}
      value={offlineCount}
      status={offlineCount ? 'red' : 'green-dark'}
      description={
        activatedCount !== undefined
          ? t('overview.sensors.description', { count: activatedCount })
          : undefined
      }
      link={{ to: '/sensors', search: { statuses: [SensorStatus.Offline] } }}
    />
  )
}

export default SensorsStatTile
