import { SensorStatus, SoilCondition, WateringStatus } from '@/api/backendApi'
import type { Tree, TreeCluster } from '@/api/backendApi'

export type UnknownStatusReasonKey =
  'soil-unknown' | 'no-sensor' | 'sensor-silent' | 'beyond-monitoring' | 'unscorable'

export interface UnknownStatusReason {
  key: UnknownStatusReasonKey
  text: string
}

/** Probe depths the backend's KA5 table calibrates (`tree::volumetric_calibration`). */
const KA5_CALIBRATED_DEPTHS = [40, 80]

/** Last lifecycle year the watermark calibration table covers. */
const MONITORED_GROWTH_YEARS = 3

const DISPLAY_ORDER: UnknownStatusReasonKey[] = [
  'soil-unknown',
  'no-sensor',
  'sensor-silent',
  'beyond-monitoring',
  'unscorable',
]

const measuresCalibratedMoisture = (tree: Tree): boolean =>
  (tree.sensor?.model.abilities ?? []).some(
    (ability) =>
      ability.ability === 'soil_moisture' && KA5_CALIBRATED_DEPTHS.includes(ability.depthCm),
  )

const measuresTension = (tree: Tree): boolean =>
  (tree.sensor?.model.abilities ?? []).some((ability) => ability.ability === 'soil_tension')

const diagnose = (
  tree: Tree,
  soilBlocksVolumetric: boolean,
  currentYear: number,
): UnknownStatusReasonKey | null => {
  // Sensorless trees are excluded from the cluster's majority vote.
  if (!tree.sensor) return null
  if (tree.sensor.status !== SensorStatus.Online) return 'sensor-silent'
  // Already covered by the cluster-wide soil reason.
  if (soilBlocksVolumetric && measuresCalibratedMoisture(tree)) return null
  if (measuresTension(tree) && currentYear - tree.plantingYear > MONITORED_GROWTH_YEARS) {
    return 'beyond-monitoring'
  }
  return 'unscorable'
}

const describe = (key: UnknownStatusReasonKey, count: number): string => {
  const trees = count === 1 ? 'Baum' : 'Bäume'
  switch (key) {
    case 'soil-unknown':
      return 'Die Bodenbeschaffenheit der Gruppe ist nicht bestimmt. Ohne sie lassen sich die Feuchtemesswerte nicht bewerten.'
    case 'no-sensor':
      return 'Diese Gruppe hat keinen Sensor, daher liegen keine Messwerte vor.'
    case 'sensor-silent':
      return count === 1
        ? '1 Sensor sendet derzeit keine Daten.'
        : `${count} Sensoren senden derzeit keine Daten.`
    case 'beyond-monitoring':
      return `${count} ${trees} liegen außerhalb des überwachten Anwuchszeitraums von ${MONITORED_GROWTH_YEARS} Jahren.`
    case 'unscorable':
      return `Für ${count} ${trees} liegen keine auswertbaren Messwerte vor.`
  }
}

/**
 * Explains why a cluster has no watering status. Empty while the status is
 * known, and for a cluster without trees, which has its own notice.
 */
export const unknownStatusReasons = (
  cluster: TreeCluster,
  currentYear: number,
): UnknownStatusReason[] => {
  const trees = cluster.trees ?? []
  if (cluster.wateringStatus !== WateringStatus.Unknown || trees.length === 0) return []

  if (!trees.some((tree) => tree.sensor)) {
    return [{ key: 'no-sensor', text: describe('no-sensor', trees.length) }]
  }

  // A missing soil type blocks the whole group, whatever its sensors do.
  const soilBlocksVolumetric =
    cluster.soilCondition === SoilCondition.Unknown && trees.some(measuresCalibratedMoisture)

  const counts = new Map<UnknownStatusReasonKey, number>()
  if (soilBlocksVolumetric) counts.set('soil-unknown', 1)
  for (const tree of trees) {
    if (tree.wateringStatus !== WateringStatus.Unknown) continue
    const key = diagnose(tree, soilBlocksVolumetric, currentYear)
    if (!key) continue
    counts.set(key, (counts.get(key) ?? 0) + 1)
  }

  return DISPLAY_ORDER.flatMap((key) => {
    const count = counts.get(key)
    return count ? [{ key, text: describe(key, count) }] : []
  })
}
