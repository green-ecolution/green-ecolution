import type { SoilMoistureDepthSeriesResponse } from '@green-ecolution/backend-client'

export interface SoilMoistureChartRow {
  ts: number
  [seriesKey: string]: number | [number, number] | undefined
}

// Okabe-Ito, matching analysis/soil_moisture_by_cluster.R: one hue per depth.
const DEPTH_COLORS: Record<number, string> = {
  40: '#0072B2',
  80: '#D55E00',
}
const FALLBACK_COLORS = ['#009E73', '#CC79A7', '#E69F00']

export const depthColor = (depthCm: number, index: number): string =>
  DEPTH_COLORS[depthCm] ?? FALLBACK_COLORS[index % FALLBACK_COLORS.length]

export const toChartRows = (series: SoilMoistureDepthSeriesResponse[]): SoilMoistureChartRow[] => {
  const byTs = new Map<number, SoilMoistureChartRow>()
  for (const depth of series) {
    for (const p of depth.points) {
      const ts = new Date(p.timestamp).getTime()
      const row = byTs.get(ts) ?? { ts }
      row[`mean_${depth.depthCm}`] = p.mean
      row[`range_${depth.depthCm}`] = [p.min, p.max]
      byTs.set(ts, row)
    }
  }
  return [...byTs.values()].sort((a, b) => a.ts - b.ts)
}
