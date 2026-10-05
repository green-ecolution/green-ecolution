import { useTranslation } from 'react-i18next'
import { Card, CardContent, CardHeader, CardTitle } from '@green-ecolution/ui'
import type { Sensor } from '@/api/backendApi'
import { parseSignal } from './signalParsing'
import ChartSignalData from './ChartSignalData'
import { compactCardContent, compactCardHeader, compactCardTitle } from './cardDensity'

interface SensorSignalCardProps {
  sensor: Sensor
}

const SensorSignalCard = ({ sensor }: SensorSignalCardProps) => {
  const { t } = useTranslation('sensor')
  const signal = sensor.sensorType === 'lorawan' ? parseSignal(sensor.latestData) : null
  // Without a reading the KPI row already says so; an empty card would only repeat it.
  if (!signal) return null

  return (
    <Card variant="outlined">
      <CardHeader className={compactCardHeader}>
        <CardTitle className={compactCardTitle}>{t('signal.title')}</CardTitle>
        <p className="flex flex-wrap gap-x-4 gap-y-1 text-sm tabular-nums">
          <span>
            <span className="text-muted-foreground">{t('signal.snrLabel')}</span>{' '}
            <span className="font-semibold">{`${signal.snrDb} dB`}</span>
          </span>
          <span className="font-semibold">
            {t('signal.gatewayCount', { count: signal.gatewayCount })}
          </span>
        </p>
      </CardHeader>
      <CardContent className={compactCardContent}>
        <ChartSignalData sensorId={sensor.id} />
      </CardContent>
    </Card>
  )
}

export default SensorSignalCard
