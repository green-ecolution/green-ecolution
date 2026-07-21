import { describe, it, expect } from 'vitest'
import type { SoilMoisturePointResponse } from '@green-ecolution/backend-client'
import { toChartRows, depthColor } from './soilMoistureChart'

const point = (
  timestamp: string,
  mean: number,
  min: number,
  max: number,
): SoilMoisturePointResponse => ({
  timestamp: new Date(timestamp),
  mean,
  min,
  max,
  sampleCount: 1,
})

describe('toChartRows', () => {
  it('merges depths sharing a bucket into one row', () => {
    const rows = toChartRows([
      { depthCm: 40, points: [point('2026-07-02T00:00:00Z', 25, 20, 30)] },
      { depthCm: 80, points: [point('2026-07-02T00:00:00Z', 18, 17, 19)] },
    ])
    expect(rows).toHaveLength(1)
    expect(rows[0].mean_40).toBe(25)
    expect(rows[0].range_40).toEqual([20, 30])
    expect(rows[0].mean_80).toBe(18)
  })

  it('keeps buckets missing in one depth and sorts by time', () => {
    const rows = toChartRows([
      { depthCm: 40, points: [point('2026-07-03T00:00:00Z', 20, 20, 20)] },
      { depthCm: 80, points: [point('2026-07-02T00:00:00Z', 18, 18, 18)] },
    ])
    expect(rows).toHaveLength(2)
    expect(rows[0].mean_80).toBe(18)
    expect(rows[0].mean_40).toBeUndefined()
    expect(rows[1].mean_40).toBe(20)
  })
})

describe('depthColor', () => {
  it('uses fixed Okabe-Ito hues for the standard depths', () => {
    expect(depthColor(40, 0)).toBe('#0072B2')
    expect(depthColor(80, 1)).toBe('#D55E00')
  })

  it('rotates fallback hues for unexpected depths', () => {
    expect(depthColor(120, 2)).toBe('#E69F00')
  })
})
