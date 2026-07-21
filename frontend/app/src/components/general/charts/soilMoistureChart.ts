import type {
  ClusterWateringEventResponse,
  SoilMoistureConditionPointResponse,
  SoilMoistureDepthSeriesResponse,
} from '@green-ecolution/backend-client'

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

export interface ConditionChartRow {
  ts: number
  supply?: number
  supplyRange?: [number, number]
}

export const toConditionRows = (
  condition: SoilMoistureConditionPointResponse[],
): ConditionChartRow[] =>
  condition
    .map((p) => ({
      ts: new Date(p.timestamp).getTime(),
      supply: p.mean,
      supplyRange: [p.min, p.max] as [number, number],
    }))
    .sort((a, b) => a.ts - b.ts)

export interface WateringEventMarker {
  wateringPlanId: string
  date: Date
  ts: number
}

// Watering events land mid-day like in the R analysis, so the marker sits
// inside the day bucket instead of on its edge. `date` is already a Date
// (midnight UTC from the "YYYY-MM-DD" wire value), so shift by 12h instead
// of round-tripping through a string.
// The last row's ts is the start of today's bucket, so a same-day event's
// noon timestamp can fall past it; widen the upper bound by one bucket
// width, then clamp (Recharts drops ReferenceLines beyond the data max).
export const wateringEventMarkers = (
  events: ClusterWateringEventResponse[],
  rows: { ts: number }[],
  bucket: 'hour' | 'day',
): WateringEventMarker[] => {
  if (rows.length <= 1) return []
  const bucketWidthMs = bucket === 'hour' ? 60 * 60 * 1000 : 24 * 60 * 60 * 1000
  const lastTs = rows[rows.length - 1].ts
  return events
    .map((event) => ({
      wateringPlanId: event.wateringPlanId,
      date: event.date,
      ts: event.date.getTime() + 12 * 60 * 60 * 1000,
    }))
    .filter((marker) => marker.ts >= rows[0].ts && marker.ts <= lastTs + bucketWidthMs)
    .map((marker) => ({ ...marker, ts: Math.min(marker.ts, lastTs) }))
}
