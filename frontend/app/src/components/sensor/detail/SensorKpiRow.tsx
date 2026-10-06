import { useTranslation } from 'react-i18next'
import { SignalBars, StatusCard, cn } from '@green-ecolution/ui'
import { useSensorStatusDetails } from '@/hooks/details/useDetailsForSensorStatus'
import { useDateLocale } from '@/lib/i18n/useFormatters'
import { formatBatteryVoltage, formatLastSeen, parseBatteryVoltage } from './latestDataParsing'
import { formatSendInterval } from './configParsing'
import {
  parseSignal,
  signalBarsFromRssi,
  signalLevelFromRssi,
  useSignalLevelLabel,
  SIGNAL_LEVEL_TEXT_COLOR,
} from './signalParsing'
import type { Sensor } from '@/api/backendApi'

interface SensorKpiRowProps {
  sensor: Sensor
}

const SensorKpiRow = ({ sensor }: SensorKpiRowProps) => {
  const { t } = useTranslation('sensor')
  const dateLocale = useDateLocale()
  const getSensorStatusDetails = useSensorStatusDetails()
  const getSignalLevelLabel = useSignalLevelLabel()
  const status = getSensorStatusDetails(sensor.status)
  const battery = parseBatteryVoltage(sensor.latestData)
  const batteryStatus =
    battery === null ? 'default' : battery < 2.8 ? 'outline-red' : 'outline-green-dark'
  const sendInterval = formatSendInterval(sensor, t)
  const isLorawan = sensor.sensorType === 'lorawan'
  const signal = isLorawan ? parseSignal(sensor.latestData) : null
  const signalLevel = signal ? signalLevelFromRssi(signal.rssiDbm) : null

  return (
    <section aria-labelledby="sensor-kpi-heading">
      <h2 id="sensor-kpi-heading" className="sr-only">
        {t('kpi.srHeading')}
      </h2>
      <ul
        className={cn(
          'grid grid-cols-2 gap-3 md:gap-4',
          isLorawan ? 'lg:grid-cols-4' : 'lg:grid-cols-3',
        )}
      >
        <li>
          <StatusCard
            size="compact"
            status={status.color}
            indicator="dot"
            label={t('kpi.statusLabel')}
            value={status.label}
            info={status.description}
          />
        </li>
        <li>
          <StatusCard
            size="compact"
            status={batteryStatus}
            label={t('kpi.batteryLabel')}
            value={
              battery === null ? t('kpi.batteryNoData') : formatBatteryVoltage(sensor.latestData)
            }
            info={t('kpi.batteryShutoffInfo')}
          />
        </li>
        <li>
          <StatusCard
            size="compact"
            label={t('kpi.lastSignalLabel')}
            value={formatLastSeen(sensor.latestData, dateLocale)}
            description={
              sendInterval && (
                <span className="text-muted-foreground">
                  {t('kpi.sendsEvery', { interval: sendInterval })}
                </span>
              )
            }
          />
        </li>
        {isLorawan && (
          <li>
            <StatusCard
              size="compact"
              indicator={signal ? 'dot' : 'none'}
              icon={
                signal &&
                signalLevel && (
                  <SignalBars
                    filled={signalBarsFromRssi(signal.rssiDbm)}
                    size="md"
                    className={SIGNAL_LEVEL_TEXT_COLOR[signalLevel]}
                  />
                )
              }
              label={t('kpi.receptionLabel')}
              value={signal ? `${signal.rssiDbm} dBm` : t('kpi.receptionNoData')}
              info={t('kpi.receptionInfo')}
              description={
                signalLevel && (
                  <span className="text-muted-foreground">{getSignalLevelLabel(signalLevel)}</span>
                )
              }
            />
          </li>
        )}
      </ul>
    </section>
  )
}

export default SensorKpiRow
