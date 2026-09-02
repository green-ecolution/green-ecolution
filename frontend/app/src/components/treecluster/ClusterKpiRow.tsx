import { format, formatDistanceToNow } from 'date-fns'
import { useTranslation } from 'react-i18next'
import { StatusCard } from '@green-ecolution/ui'
import StatusCardGrid from '@/components/general/StatusCardGrid'
import { useWateringStatusDetails } from '@/hooks/details/useDetailsForWateringStatus'
import { roundTo } from '@/lib/utils'
import { useDateLocale } from '@/lib/i18n/useFormatters'
import { latestClusterReading } from './clusterLatestReading'
import type { TreeCluster } from '@/api/backendApi'

interface ClusterKpiRowProps {
  treecluster: TreeCluster
}

const ClusterKpiRow = ({ treecluster }: ClusterKpiRowProps) => {
  const { t } = useTranslation('treecluster')
  const dateLocale = useDateLocale()
  const getWateringStatusDetails = useWateringStatusDetails()
  const wateringStatus = getWateringStatusDetails(treecluster.wateringStatus)
  const { temperature, measuredAt } = latestClusterReading(treecluster.trees ?? [])

  return (
    <StatusCardGrid columns={4}>
      <li className="h-full">
        <StatusCard
          size="compact"
          status={wateringStatus.color}
          indicator="dot"
          label={t('kpi.wateringStatusLabel')}
          value={wateringStatus.label}
          info={wateringStatus.description}
        />
      </li>
      <li className="h-full">
        <StatusCard
          size="compact"
          label={t('kpi.soilTemperatureLabel')}
          value={temperature != null ? `${roundTo(temperature, 1)} °C` : t('kpi.noData')}
        />
      </li>
      <li className="h-full">
        <StatusCard
          size="compact"
          label={t('kpi.lastMeasurementLabel')}
          value={
            measuredAt
              ? formatDistanceToNow(measuredAt, { addSuffix: true, locale: dateLocale })
              : t('kpi.noData')
          }
        />
      </li>
      <li className="h-full">
        <StatusCard
          size="compact"
          label={t('kpi.lastWateredLabel')}
          value={
            treecluster.lastWatered
              ? format(new Date(treecluster.lastWatered), 'dd.MM.yyyy', { locale: dateLocale })
              : '—'
          }
          info={t('kpi.lastWateredInfo')}
        />
      </li>
    </StatusCardGrid>
  )
}

export default ClusterKpiRow
