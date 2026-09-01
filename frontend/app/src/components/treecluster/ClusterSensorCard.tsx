import {
  Card,
  CardContent,
  CardHeader,
  CardTitle,
  DetailedList,
  InlineAlert,
  SignalBars,
} from '@green-ecolution/ui'
import { Link } from '@tanstack/react-router'
import { useTranslation } from 'react-i18next'
import type { TFunction } from 'i18next'
import type { Tree } from '@/api/backendApi'
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

  return (
    <Link
      to="/sensors/$sensorId"
      params={{ sensorId: sensor.id }}
      className="block rounded-lg border border-dark-50 bg-white p-4 transition-colors hover:border-green-dark"
    >
      <p className="mb-2 font-lato text-lg font-bold">
        {t('sensorCard.treeLabel', { species: tree.species, number: tree.number })}
      </p>
      {hasQualityWarning(sensor) && (
        <InlineAlert
          variant={getDataQualityDetails(sensor).alert}
          description={getDataQualityDetails(sensor).label}
          className="mb-3"
        />
      )}
      <DetailedList
        columns={1}
        details={[
          {
            label: t('sensorCard.signalLabel'),
            value: signal ? (
              <span
                className={`flex items-center gap-2 ${SIGNAL_LEVEL_TEXT_COLOR[signalLevelFromRssi(signal.rssiDbm)]}`}
              >
                <SignalBars filled={signalBarsFromRssi(signal.rssiDbm)} />
                {getSignalLevelLabel(signalLevelFromRssi(signal.rssiDbm))}
              </span>
            ) : (
              t('sensorCard.noSignalData')
            ),
          },
          { label: t('sensorCard.batteryLabel'), value: formatBatteryVoltage(sensor.latestData) },
          {
            label: t('sensorCard.lastTransmissionLabel'),
            value: formatLastSeen(sensor.latestData, dateLocale),
          },
        ]}
      />
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
            <div className="flex flex-col gap-y-3">
              {treesWithSensor.map((tree) => (
                <SensorTreeRow key={tree.id} tree={tree} t={t} />
              ))}
            </div>
            <p className="mt-4 text-right text-sm text-muted-foreground tabular-nums">
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
