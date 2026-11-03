import { Badge, Card, CardContent, CardHeader, CardTitle, SignalBars } from '@green-ecolution/ui'
import { Link } from '@tanstack/react-router'
import { useTranslation } from 'react-i18next'
import type { TFunction } from 'i18next'
import SensorIcon from '@/components/icons/Sensor'
import type { Tree } from '../../api/client'
import {
  parseSignal,
  signalBarsFromRssi,
  signalLevelFromRssi,
  useSignalLevelLabel,
  SIGNAL_LEVEL_TEXT_COLOR,
} from '@/components/sensor/detail/signalParsing'
import { formatBatteryVoltage, formatLastSeen } from '@/components/sensor/detail/latestDataParsing'
import { useDataQualityDetails, hasQualityWarning } from '@/hooks/details/useDetailsForDataHealth'
import { useDateLocale } from '@/lib/i18n/useFormatters'

interface ClusterSensorCardProps {
  trees: Tree[]
}

const SensorTreeRow = ({ tree, t }: { tree: Tree; t: TFunction<'treecluster'> }) => {
  const dateLocale = useDateLocale()
  const getDataQualityDetails = useDataQualityDetails()
  const getSignalLevelLabel = useSignalLevelLabel()
  const sensor = tree.sensor
  if (!sensor) return null

  const signal = parseSignal(sensor.latestData)
  const signalLevel = signal ? signalLevelFromRssi(signal.rssiDbm) : null
  const quality = hasQualityWarning(sensor) ? getDataQualityDetails(sensor) : null

  return (
    <Link
      to="/sensors/$sensorId"
      params={{ sensorId: sensor.id }}
      className="group flex gap-3 rounded-sm py-3 text-sm focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-green-dark"
    >
      <SensorIcon className="mt-0.5 size-4 shrink-0 text-green-dark" aria-hidden />
      <div className="min-w-0 flex-1">
        <p className="truncate group-hover:underline">
          <span className="font-semibold">{tree.species}</span>
          <span className="text-dark-600">
            {' · '}
            {t('treeList.number', { number: tree.number })}
          </span>
        </p>
        <p className="mt-1 flex flex-wrap items-center gap-x-1.5 text-xs text-muted-foreground">
          <span
            title={t('sensorCard.signalLabel')}
            className={`flex items-center gap-1.5 ${signalLevel ? SIGNAL_LEVEL_TEXT_COLOR[signalLevel] : ''}`}
          >
            <span className="sr-only">{t('sensorCard.signalLabel')}:</span>
            {signal && signalLevel && <SignalBars filled={signalBarsFromRssi(signal.rssiDbm)} />}
            {signalLevel ? getSignalLevelLabel(signalLevel) : t('sensorCard.noSignalData')}
          </span>
          <span aria-hidden>·</span>
          <span title={t('sensorCard.batteryLabel')} className="tabular-nums">
            <span className="sr-only">{t('sensorCard.batteryLabel')}:</span>
            {formatBatteryVoltage(sensor.latestData)}
          </span>
          <span aria-hidden>·</span>
          <span title={t('sensorCard.lastTransmissionLabel')}>
            <span className="sr-only">{t('sensorCard.lastTransmissionLabel')}:</span>
            {formatLastSeen(sensor.latestData, dateLocale)}
          </span>
        </p>
        {quality && (
          <Badge variant={quality.color} className="mt-2 gap-1.5">
            <span className="size-1.5 rounded-full bg-current" aria-hidden />
            {quality.label}
          </Badge>
        )}
      </div>
    </Link>
  )
}

const ClusterSensorCard = ({ trees }: ClusterSensorCardProps) => {
  const { t } = useTranslation('treecluster')
  const treesWithSensor = trees.filter((tree) => tree.sensor)

  return (
    <Card variant="outlined">
      <CardHeader>
        <CardTitle>{t('sensorCard.title')}</CardTitle>
      </CardHeader>
      <CardContent>
        {treesWithSensor.length === 0 ? (
          <p className="text-sm text-muted-foreground">{t('sensorCard.noSensorNotice')}</p>
        ) : (
          <>
            <ul className="divide-y divide-dark-100">
              {treesWithSensor.map((tree) => (
                <li key={tree.id}>
                  <SensorTreeRow tree={tree} t={t} />
                </li>
              ))}
            </ul>
            <p className="mt-3 text-right text-xs text-muted-foreground tabular-nums">
              {t('sensorCard.sensorCount', {
                count: trees.length,
                withSensor: treesWithSensor.length,
              })}
            </p>
          </>
        )}
      </CardContent>
    </Card>
  )
}

export default ClusterSensorCard
