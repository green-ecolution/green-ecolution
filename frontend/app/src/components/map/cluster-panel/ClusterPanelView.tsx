import { MoveRight, RadioTower } from 'lucide-react'
import { format, formatDistanceToNow } from 'date-fns'
import { useTranslation } from 'react-i18next'
import { Badge, Button, StatusCard } from '@green-ecolution/ui'
import { useWateringStatusDetails } from '@/hooks/details/useDetailsForWateringStatus'
import { useDateLocale } from '@/lib/i18n/useFormatters'
import { roundTo } from '@/lib/utils'
import Tree from '@/components/icons/Tree'
import type { SensorPayload, TreeClusterResponse } from '@/api/backendApi'
import { latestSensorReading, sortTreesSensorFirst, summarizeTopSpecies } from './clusterPanelUtils'

interface ClusterPanelViewProps {
  treecluster: TreeClusterResponse
  onOpenDashboard: () => void
}

const PREVIEW_COUNT = 3

const ClusterPanelView = ({ treecluster, onOpenDashboard }: ClusterPanelViewProps) => {
  const { t } = useTranslation(['map', 'common'])
  const dateLocale = useDateLocale()
  const getWateringStatusDetails = useWateringStatusDetails()
  const status = getWateringStatusDetails(treecluster.wateringStatus)
  const species = summarizeTopSpecies(treecluster.trees)
  const sortedTrees = sortTreesSensorFirst(treecluster.trees)
  const previewTrees = sortedTrees.slice(0, PREVIEW_COUNT)
  const remaining = sortedTrees.length - previewTrees.length
  const treeCount = treecluster.trees.length
  const reading = latestSensorReading(treecluster.trees)
  const temperatureValue = (reading?.data as SensorPayload | undefined)?.temperature
  const temperature =
    typeof temperatureValue === 'number'
      ? `${roundTo(temperatureValue, 1)} °C`
      : t('cluster.noData')
  const lastMeasurement = reading
    ? formatDistanceToNow(new Date(reading.createdAt), { addSuffix: true, locale: dateLocale })
    : t('cluster.noData')
  const lastWatered = treecluster.lastWatered
    ? format(new Date(treecluster.lastWatered), 'dd.MM.yyyy')
    : t('common:state.noData')

  return (
    <div className="flex flex-col gap-y-5">
      <p className="text-sm text-dark-600">
        {treecluster.address} · {t('cluster.treeCount', { count: treeCount })}
        {species && ` · ${species}`}
      </p>

      <Button onClick={onOpenDashboard} className="group w-full lg:order-last">
        {t('cluster.openDashboard')}
        <MoveRight className="icon-arrow-animate" />
      </Button>

      <div className="grid grid-cols-2 gap-4">
        <StatusCard
          status={status.color}
          indicator="dot"
          label={t('cluster.wateringStatusLabel')}
          value={status.label}
          description={status.description}
        />
        <StatusCard label={t('cluster.soilTemperatureLabel')} value={temperature} isLarge />
        <StatusCard label={t('cluster.lastMeasurementLabel')} value={lastMeasurement} />
        <StatusCard label={t('cluster.lastWateredLabel')} value={lastWatered} />
      </div>

      <section>
        <p className="mb-2 text-xs font-semibold uppercase tracking-wide text-dark-500">
          {t('cluster.treesInGroupHeading', { count: treeCount })}
        </p>
        <ul className="flex flex-col">
          {previewTrees.map((tree) => (
            <li
              key={tree.id}
              data-testid="cluster-panel-tree"
              className="flex items-center gap-3 border-b border-dark-100 py-3 text-sm last:border-0"
            >
              {tree.sensor ? (
                <RadioTower className="size-4 shrink-0 text-green-dark" />
              ) : (
                <Tree className="size-4 shrink-0 text-dark-400" />
              )}
              <span className="min-w-0 flex-1 truncate">
                <span className="font-medium text-dark-800">{tree.species}</span>
                <span className="text-dark-500"> · ID {tree.number}</span>
              </span>
              {tree.sensor ? (
                <Badge variant="success" className="shrink-0">
                  {t('cluster.sensorTreeBadge')}
                </Badge>
              ) : (
                <span className="shrink-0 text-dark-500">{t('cluster.noSensor')}</span>
              )}
            </li>
          ))}
        </ul>
        {remaining > 0 && (
          <button
            type="button"
            onClick={onOpenDashboard}
            className="mt-3 cursor-pointer text-sm font-semibold text-green-dark transition-colors hover:text-green-dark-700"
          >
            {t('cluster.remainingTrees', { count: remaining })}
          </button>
        )}
      </section>
    </div>
  )
}

export default ClusterPanelView
