import { describe, expect, it, beforeAll } from 'vitest'
import type { TFunction } from 'i18next'
import { getI18n } from '@/lib/i18n'
import type { Sensor } from '@/api/backendApi'
import { formatSendInterval } from './configParsing'

const lorawanSensor = (config: Record<string, unknown> | null): Sensor =>
  ({ sensorType: 'lorawan', lorawan: config ? { config } : {} }) as unknown as Sensor

// vitest setup already calls createI18n(); fix the language so assertions
// against the German catalog text stay stable regardless of test order.
let t: TFunction<'sensor'>
beforeAll(() => {
  t = getI18n().getFixedT('de', 'sensor')
})

describe('formatSendInterval', () => {
  it('formats sub-minute intervals in seconds', () => {
    expect(formatSendInterval(lorawanSensor({ TDC: '30000' }), t)).toBe('alle 30 Sek.')
  })

  it('formats minute intervals', () => {
    expect(formatSendInterval(lorawanSensor({ TDC: '60000' }), t)).toBe('alle 1 Min.')
    expect(formatSendInterval(lorawanSensor({ TDC: '120000' }), t)).toBe('alle 2 Min.')
  })

  it('formats hour intervals', () => {
    expect(formatSendInterval(lorawanSensor({ TDC: '3600000' }), t)).toBe('alle 1 Std.')
  })

  it('accepts a numeric TDC value', () => {
    expect(formatSendInterval(lorawanSensor({ TDC: 300000 }), t)).toBe('alle 5 Min.')
  })

  it('returns null when TDC is missing', () => {
    expect(formatSendInterval(lorawanSensor({ OTAA: '1' }), t)).toBeNull()
    expect(formatSendInterval(lorawanSensor(null), t)).toBeNull()
  })

  it('returns null for invalid or non-positive TDC', () => {
    expect(formatSendInterval(lorawanSensor({ TDC: 'abc' }), t)).toBeNull()
    expect(formatSendInterval(lorawanSensor({ TDC: '0' }), t)).toBeNull()
    expect(formatSendInterval(lorawanSensor({ TDC: '-1000' }), t)).toBeNull()
  })

  it('returns null for non-lorawan sensors', () => {
    const sensor = {
      sensorType: 'other',
      lorawan: { config: { TDC: '60000' } },
    } as unknown as Sensor
    expect(formatSendInterval(sensor, t)).toBeNull()
  })
})
