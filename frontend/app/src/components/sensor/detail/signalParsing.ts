import type { SensorData } from '@/api/backendApi'

export type SignalLevel = 'good' | 'fair' | 'weak'

export interface ParsedSignal {
  rssiDbm: number
  snrDb: number
  gatewayCount: number
}

export const parseSignal = (latestData: SensorData | null | undefined): ParsedSignal | null => {
  const s = latestData?.signal
  if (!s || typeof s.rssiDbm !== 'number') return null
  return {
    rssiDbm: s.rssiDbm,
    snrDb: typeof s.snrDb === 'number' ? s.snrDb : 0,
    gatewayCount: typeof s.gatewayCount === 'number' ? s.gatewayCount : 0,
  }
}

export const signalBarsFromRssi = (rssiDbm: number): number => {
  if (rssiDbm >= -95) return 4
  if (rssiDbm >= -105) return 3
  if (rssiDbm >= -115) return 2
  if (rssiDbm >= -120) return 1
  return 0
}

export const signalLevelFromRssi = (rssiDbm: number): SignalLevel => {
  const bars = signalBarsFromRssi(rssiDbm)
  if (bars >= 3) return 'good'
  if (bars === 2) return 'fair'
  return 'weak'
}

export const SIGNAL_LEVEL_LABEL: Record<SignalLevel, string> = {
  good: 'Gut',
  fair: 'Ausreichend',
  weak: 'Schwach',
}

export const SIGNAL_LEVEL_TEXT_COLOR: Record<SignalLevel, string> = {
  good: 'text-green-dark',
  fair: 'text-yellow-900',
  weak: 'text-red',
}
