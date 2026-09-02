import type { Locale } from 'date-fns'
import type { SensorDataResponse } from '@green-ecolution/backend-client'
import { formatDistanceToNow } from 'date-fns'

type SensorData = Record<string, unknown>

const asData = (latestData: SensorDataResponse | null | undefined): SensorData | null => {
  if (!latestData?.data) return null
  return latestData.data as SensorData
}

export const parseBatteryVoltage = (
  latestData: SensorDataResponse | null | undefined,
): number | null => {
  const data = asData(latestData)
  const v = data?.battery
  return typeof v === 'number' ? v : null
}

export const formatBatteryVoltage = (latestData: SensorDataResponse | null | undefined): string => {
  const v = parseBatteryVoltage(latestData)
  return v === null ? '-' : `${v.toFixed(2)} V`
}

// Not a component: the caller resolves the active locale via `useDateLocale()`
// and passes it in, rather than this module reaching for `getI18n()` — a
// frozen locale here would reproduce the router-helper bug fixed earlier.
export const formatLastSeen = (
  latestData: SensorDataResponse | null | undefined,
  locale: Locale,
): string => {
  const ts = latestData?.updatedAt ?? latestData?.createdAt
  if (!ts) return '-'
  try {
    return formatDistanceToNow(new Date(ts), { locale, addSuffix: true })
  } catch {
    return '-'
  }
}
