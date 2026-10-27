import { describe, it, expect } from 'vitest'
import type {
  ClusterWateringEventResponse,
  SoilMoistureConditionPointResponse,
} from '@green-ecolution/backend-client'
import { toConditionRows, wateringEventMarkers } from './soilMoistureChart'

const conditionPoint = (
  timestamp: string,
  mean: number,
  min: number,
  max: number,
): SoilMoistureConditionPointResponse => ({
  timestamp: new Date(timestamp),
  mean,
  min,
  max,
  worstDepthCm: 80,
})

describe('toConditionRows', () => {
  it('maps points to rows sorted by time', () => {
    const rows = toConditionRows([
      conditionPoint('2026-07-03T00:00:00Z', 20, 18, 22),
      conditionPoint('2026-07-02T00:00:00Z', 40, 35, 45),
    ])
    expect(rows).toHaveLength(2)
    expect(rows[0].supply).toBe(40)
    expect(rows[0].supplyRange).toEqual([35, 45])
    expect(rows[1].supply).toBe(20)
    expect(rows[0].ts).toBeLessThan(rows[1].ts)
  })
})

describe('wateringEventMarkers', () => {
  const event = (id: string, date: string): ClusterWateringEventResponse => ({
    wateringPlanId: id,
    date: new Date(date),
    consumedWaterLiters: 100,
  })
  const rows = [
    { ts: Date.parse('2026-07-01T00:00:00Z') },
    { ts: Date.parse('2026-07-02T00:00:00Z') },
  ]

  it('shifts events to noon and keeps in-window ones', () => {
    const markers = wateringEventMarkers([event('a', '2026-07-01')], rows, 'day')
    expect(markers).toHaveLength(1)
    expect(markers[0].ts).toBe(Date.parse('2026-07-01T12:00:00Z'))
  })

  it('clamps a same-day event to the last row instead of dropping it', () => {
    const markers = wateringEventMarkers([event('b', '2026-07-02')], rows, 'day')
    expect(markers).toHaveLength(1)
    expect(markers[0].ts).toBe(rows[1].ts)
  })

  it('drops out-of-window events and returns none for short series', () => {
    expect(wateringEventMarkers([event('c', '2026-06-20')], rows, 'day')).toHaveLength(0)
    expect(wateringEventMarkers([event('d', '2026-07-01')], [rows[0]], 'day')).toHaveLength(0)
  })
})
