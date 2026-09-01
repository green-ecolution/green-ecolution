import type { TFunction } from 'i18next'
import type { Sensor } from '@/api/backendApi'

// LoRaWAN devices report their transmit interval via the TDC AT-command (milliseconds).
export const formatSendInterval = (sensor: Sensor, t: TFunction<'sensor'>): string | null => {
  if (sensor.sensorType !== 'lorawan') return null

  const config = sensor.lorawan?.config as Record<string, unknown> | undefined
  const ms = Number(config?.TDC)
  if (!Number.isFinite(ms) || ms <= 0) return null

  const seconds = ms / 1000
  if (seconds < 60) return t('sendInterval.seconds', { count: Math.round(seconds) })

  const minutes = seconds / 60
  if (minutes < 60) return t('sendInterval.minutes', { count: Math.round(minutes) })

  return t('sendInterval.hours', { count: Math.round(minutes / 60) })
}
