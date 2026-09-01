import { useTranslation } from 'react-i18next'
import { StatusCard } from '@green-ecolution/ui'
import { useSensorStatusDetails } from '@/hooks/details/useDetailsForSensorStatus'
import { useDateLocale } from '@/lib/i18n/useFormatters'
import { formatBatteryVoltage, formatLastSeen, parseBatteryVoltage } from './latestDataParsing'
import { formatSendInterval } from './configParsing'
import type { Sensor } from '@/api/backendApi'

interface SensorStatusGridProps {
  sensor: Sensor
}

const SensorStatusGrid = ({ sensor }: SensorStatusGridProps) => {
  const { t } = useTranslation('sensor')
  const dateLocale = useDateLocale()
  const getSensorStatusDetails = useSensorStatusDetails()
  const status = getSensorStatusDetails(sensor.status)
  const battery = parseBatteryVoltage(sensor.latestData)
  const batteryStatus =
    battery === null ? 'default' : battery < 2.8 ? 'outline-red' : 'outline-green-dark'
  const sendInterval = formatSendInterval(sensor, t)

  return (
    <section aria-labelledby="sensor-status-heading">
      <h2 id="sensor-status-heading" className="sr-only">
        {t('statusGrid.srHeading')}
      </h2>
      <ul className="grid gap-4 md:grid-cols-3">
        <li>
          <StatusCard
            status={status.color}
            indicator="dot"
            label={t('statusGrid.statusLabel')}
            value={status.label}
            description={status.description}
          />
        </li>
        <li>
          <StatusCard
            status={batteryStatus}
            label={t('statusGrid.batteryLabel')}
            value={formatBatteryVoltage(sensor.latestData)}
            isLarge
            description={
              battery === null
                ? t('statusGrid.batteryNoDataDescription')
                : t('statusGrid.batteryShutoffDescription')
            }
          />
        </li>
        <li>
          <StatusCard
            label={t('statusGrid.lastSignalLabel')}
            value={formatLastSeen(sensor.latestData, dateLocale)}
            description={
              sendInterval
                ? t('statusGrid.lastSignalDescriptionWithInterval', { interval: sendInterval })
                : t('statusGrid.lastSignalDescription')
            }
          />
        </li>
      </ul>
    </section>
  )
}

export default SensorStatusGrid
