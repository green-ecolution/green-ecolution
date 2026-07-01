import { describe, expect, it } from 'vitest'
import type { Sensor } from '@/api/backendApi'
import { formatSendInterval } from './configParsing'

const lorawanSensor = (config: Record<string, unknown> | null): Sensor =>
  ({ sensorType: 'lorawan', lorawan: config ? { config } : {} }) as unknown as Sensor

describe('formatSendInterval', () => {
  it('formats sub-minute intervals in seconds', () => {
    expect(formatSendInterval(lorawanSensor({ TDC: '30000' }))).toBe('alle 30 Sek.')
  })

  it('formats minute intervals', () => {
    expect(formatSendInterval(lorawanSensor({ TDC: '60000' }))).toBe('alle 1 Min.')
    expect(formatSendInterval(lorawanSensor({ TDC: '120000' }))).toBe('alle 2 Min.')
  })

  it('formats hour intervals', () => {
    expect(formatSendInterval(lorawanSensor({ TDC: '3600000' }))).toBe('alle 1 Std.')
  })

  it('accepts a numeric TDC value', () => {
    expect(formatSendInterval(lorawanSensor({ TDC: 300000 }))).toBe('alle 5 Min.')
  })

  it('returns null when TDC is missing', () => {
    expect(formatSendInterval(lorawanSensor({ OTAA: '1' }))).toBeNull()
    expect(formatSendInterval(lorawanSensor(null))).toBeNull()
  })

  it('returns null for invalid or non-positive TDC', () => {
    expect(formatSendInterval(lorawanSensor({ TDC: 'abc' }))).toBeNull()
    expect(formatSendInterval(lorawanSensor({ TDC: '0' }))).toBeNull()
    expect(formatSendInterval(lorawanSensor({ TDC: '-1000' }))).toBeNull()
  })

  it('returns null for non-lorawan sensors', () => {
    const sensor = {
      sensorType: 'other',
      lorawan: { config: { TDC: '60000' } },
    } as unknown as Sensor
    expect(formatSendInterval(sensor)).toBeNull()
  })
})
