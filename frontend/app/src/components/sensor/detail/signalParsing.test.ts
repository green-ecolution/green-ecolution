import { describe, expect, it } from 'vitest'
import type { SensorData } from '@/api/backendApi'
import { parseSignal, signalBarsFromRssi, signalLevelFromRssi } from './signalParsing'

const withSignal = (rssiDbm: number) =>
  ({ signal: { rssiDbm, snrDb: 0, gatewayCount: 1 } } as unknown as SensorData)

describe('signalParsing', () => {
  it('parses signal from latestData', () => {
    expect(parseSignal(withSignal(-104))).toEqual({ rssiDbm: -104, snrDb: 0, gatewayCount: 1 })
  })

  it('returns null when signal missing', () => {
    expect(parseSignal({} as unknown as SensorData)).toBeNull()
    expect(parseSignal(null)).toBeNull()
  })

  it('maps rssi to bars', () => {
    expect(signalBarsFromRssi(-90)).toBe(4)
    expect(signalBarsFromRssi(-100)).toBe(3)
    expect(signalBarsFromRssi(-110)).toBe(2)
    expect(signalBarsFromRssi(-118)).toBe(1)
    expect(signalBarsFromRssi(-125)).toBe(0)
  })

  it('maps rssi to level', () => {
    expect(signalLevelFromRssi(-100)).toBe('good')
    expect(signalLevelFromRssi(-110)).toBe('fair')
    expect(signalLevelFromRssi(-118)).toBe('weak')
  })
})
