import {
  Alert,
  AlertContent,
  AlertDescription,
  AlertIcon,
  Badge,
  Card,
  CardContent,
  CardHeader,
  CardTitle,
  DetailedList,
  SignalBars,
} from '@green-ecolution/ui'
import { Link } from '@tanstack/react-router'
import { useTranslation } from 'react-i18next'
import type { Tree } from '@/api/backendApi'
import { getSensorStatusDetails } from '@/hooks/details/useDetailsForSensorStatus'
import { getDataQualityDetails, hasQualityWarning } from '@/hooks/details/useDetailsForDataHealth'
import {
  parseSignal,
  signalBarsFromRssi,
  signalLevelFromRssi,
  SIGNAL_LEVEL_LABEL,
  SIGNAL_LEVEL_TEXT_COLOR,
} from '@/components/sensor/detail/signalParsing'
import { formatBatteryVoltage, formatLastSeen } from '@/components/sensor/detail/latestDataParsing'

interface TreeSensorCardProps {
  tree: Tree
}

const TreeSensorCard = ({ tree }: TreeSensorCardProps) => {
  const { t } = useTranslation('tree')
  const sensor = tree.sensor
  const statusDetails = sensor ? getSensorStatusDetails(sensor.status) : null
  const signal = parseSignal(sensor?.latestData)

  return (
    <Card variant="outlined">
      <CardHeader className="flex flex-row items-center justify-between gap-3">
        <CardTitle>{t('sensorCard.title')}</CardTitle>
        {statusDetails && <Badge variant={statusDetails.color}>{statusDetails.label}</Badge>}
      </CardHeader>
      <CardContent>
        {!sensor ? (
          <p className="text-sm text-muted-foreground">{t('sensorCard.noSensor')}</p>
        ) : (
          <>
            {hasQualityWarning(sensor) && (
              <Alert
                variant={getDataQualityDetails(sensor).alert}
                className="mb-3 flex w-full items-start gap-3"
              >
                <AlertIcon variant={getDataQualityDetails(sensor).alert} />
                <AlertContent>
                  <AlertDescription>{getDataQualityDetails(sensor).description}</AlertDescription>
                </AlertContent>
              </Alert>
            )}
            <Link
              to="/sensors/$sensorId"
              params={{ sensorId: sensor.id }}
              className="block rounded-lg border border-dark-50 bg-white p-4 transition-colors hover:border-green-dark"
            >
              <p className="mb-2 font-lato text-lg font-bold break-all">{sensor.id}</p>
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
                        {SIGNAL_LEVEL_LABEL[signalLevelFromRssi(signal.rssiDbm)]}
                      </span>
                    ) : (
                      t('sensorCard.noSignalData')
                    ),
                  },
                  {
                    label: t('sensorCard.batteryLabel'),
                    value: formatBatteryVoltage(sensor.latestData),
                  },
                  {
                    label: t('sensorCard.lastTransmissionLabel'),
                    value: formatLastSeen(sensor.latestData),
                  },
                ]}
              />
            </Link>
          </>
        )}
      </CardContent>
    </Card>
  )
}

export default TreeSensorCard
