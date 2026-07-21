import type { SensorData, Tree } from '@/api/backendApi'

export interface ClusterLatestReading {
  temperature: number | null
  measuredAt: Date | null
}

interface GenericReading {
  ability: string
  depth: number
  value: number
}

/** Newest reading across all cluster sensors; handles both payload shapes
 *  (legacy EcoDrizzler `temperature` field vs. generic `readings` array). */
export const latestClusterReading = (trees: Tree[]): ClusterLatestReading => {
  let latest: SensorData | null = null
  for (const tree of trees) {
    const data = tree.sensor?.latestData
    if (data && (!latest || new Date(data.updatedAt) > new Date(latest.updatedAt))) {
      latest = data
    }
  }
  if (!latest) return { temperature: null, measuredAt: null }
  const payload = latest.data as Record<string, unknown>
  const readings = payload.readings as GenericReading[] | undefined
  const temperature =
    typeof payload.temperature === 'number'
      ? payload.temperature
      : (readings?.find((r) => r.ability === 'temperature')?.value ?? null)
  return { temperature, measuredAt: new Date(latest.updatedAt) }
}
