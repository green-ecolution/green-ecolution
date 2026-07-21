import { format, formatDistanceToNow } from 'date-fns'
import { de } from 'date-fns/locale'
import { StatusCard } from '@green-ecolution/ui'
import StatusCardGrid from '@/components/general/StatusCardGrid'
import { getWateringStatusDetails } from '@/hooks/details/useDetailsForWateringStatus'
import { roundTo } from '@/lib/utils'
import { latestClusterReading } from './clusterLatestReading'
import type { TreeCluster } from '@/api/backendApi'

interface ClusterKpiRowProps {
  treecluster: TreeCluster
}

const ClusterKpiRow = ({ treecluster }: ClusterKpiRowProps) => {
  const wateringStatus = getWateringStatusDetails(treecluster.wateringStatus)
  const { temperature, measuredAt } = latestClusterReading(treecluster.trees ?? [])

  return (
    <StatusCardGrid columns={4}>
      <li className="h-full">
        <StatusCard
          size="compact"
          status={wateringStatus.color}
          indicator="dot"
          label="Bewässerungszustand (ø)"
          value={wateringStatus.label}
          info={wateringStatus.description}
        />
      </li>
      <li className="h-full">
        <StatusCard
          size="compact"
          label="Bodentemperatur"
          value={temperature != null ? `${roundTo(temperature, 1)} °C` : 'Keine Daten'}
        />
      </li>
      <li className="h-full">
        <StatusCard
          size="compact"
          label="Letzte Messung"
          value={
            measuredAt
              ? formatDistanceToNow(measuredAt, { addSuffix: true, locale: de })
              : 'Keine Daten'
          }
        />
      </li>
      <li className="h-full">
        <StatusCard
          size="compact"
          label="Letzte Bewässerung"
          value={
            treecluster.lastWatered ? format(new Date(treecluster.lastWatered), 'dd.MM.yyyy') : '—'
          }
          info="Wird aktualisiert, sobald ein Einsatzplan mit dieser Gruppe als »Beendet« markiert wird."
        />
      </li>
    </StatusCardGrid>
  )
}

export default ClusterKpiRow
