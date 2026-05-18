import type { SensorDataResponse } from '@green-ecolution/backend-client'
import { formatDistanceToNow } from 'date-fns'
import { de } from 'date-fns/locale'

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

export const formatLastSeen = (latestData: SensorDataResponse | null | undefined): string => {
  const ts = latestData?.updatedAt ?? latestData?.createdAt
  if (!ts) return '-'
  try {
    return formatDistanceToNow(new Date(ts), { locale: de, addSuffix: true })
  } catch {
    return '-'
  }
}
