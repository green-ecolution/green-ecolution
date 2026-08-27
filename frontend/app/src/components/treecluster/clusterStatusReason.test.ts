import { describe, it, expect } from 'vitest'
import { unknownStatusReasons } from './clusterStatusReason'
import type { SensorModelAbilityResponse, TreeCluster } from '@/api/backendApi'

const CURRENT_YEAR = 2026

const moistureAt = (depthCm: number): SensorModelAbilityResponse => ({
  id: `ability-moisture-${depthCm}`,
  ability: 'soil_moisture',
  unit: 'percent',
  depthCm,
})

const tensionAt = (depthCm: number): SensorModelAbilityResponse => ({
  id: `ability-tension-${depthCm}`,
  ability: 'soil_tension',
  unit: 'centibar',
  depthCm,
})

interface TreeOptions {
  wateringStatus?: string
  plantingYear?: number
  sensor?: { status: string; abilities: SensorModelAbilityResponse[] } | null
}

const makeTree = (id: string, options: TreeOptions = {}) =>
  ({
    id,
    species: 'Ahorn',
    number: `B-${id}`,
    wateringStatus: options.wateringStatus ?? 'unknown',
    plantingYear: options.plantingYear ?? CURRENT_YEAR,
    sensor: options.sensor
      ? {
          id: `sensor-${id}`,
          status: options.sensor.status,
          model: { id: 'model-1', name: 'Testmodell', abilities: options.sensor.abilities },
        }
      : null,
  }) as unknown as TreeCluster['trees'][number]

const makeCluster = (overrides: Partial<TreeCluster> = {}) =>
  ({
    id: 'cluster-1',
    name: 'Gruppe A',
    wateringStatus: 'unknown',
    soilCondition: 'Uu',
    trees: [],
    ...overrides,
  }) as unknown as TreeCluster

const keysOf = (cluster: TreeCluster) =>
  unknownStatusReasons(cluster, CURRENT_YEAR).map((reason) => reason.key)

describe('unknownStatusReasons', () => {
  it('stays silent while the status is known', () => {
    const cluster = makeCluster({
      wateringStatus: 'good',
      trees: [makeTree('a')],
    })

    expect(unknownStatusReasons(cluster, CURRENT_YEAR)).toEqual([])
  })

  it('stays silent for a cluster without trees, which has its own notice', () => {
    expect(keysOf(makeCluster())).toEqual([])
  })

  it('reports a group without any sensor', () => {
    const cluster = makeCluster({
      trees: [makeTree('a'), makeTree('b')],
    })

    const reasons = unknownStatusReasons(cluster, CURRENT_YEAR)
    expect(reasons).toHaveLength(1)
    expect(reasons[0].key).toBe('no-sensor')
    expect(reasons[0].text).toBe(
      'Diese Gruppe hat keinen Sensor, daher liegen keine Messwerte vor.',
    )
  })

  it('does not count sensorless trees once the group has any sensor', () => {
    const cluster = makeCluster({
      trees: [
        makeTree('a'),
        makeTree('b'),
        makeTree('c', { sensor: { status: 'offline', abilities: [moistureAt(40)] } }),
      ],
    })

    const reasons = unknownStatusReasons(cluster, CURRENT_YEAR)
    expect(reasons).toHaveLength(1)
    expect(reasons[0].key).toBe('sensor-silent')
  })

  it('reports a silent sensor', () => {
    const cluster = makeCluster({
      trees: [makeTree('a', { sensor: { status: 'offline', abilities: [moistureAt(40)] } })],
    })

    const reasons = unknownStatusReasons(cluster, CURRENT_YEAR)
    expect(reasons).toHaveLength(1)
    expect(reasons[0].key).toBe('sensor-silent')
    expect(reasons[0].text).toBe('1 Sensor sendet derzeit keine Daten.')
  })

  it('blames the missing soil type only for a volumetric probe', () => {
    const cluster = makeCluster({
      soilCondition: 'unknown',
      trees: [makeTree('a', { sensor: { status: 'online', abilities: [moistureAt(40)] } })],
    })

    const reasons = unknownStatusReasons(cluster, CURRENT_YEAR)
    expect(reasons).toHaveLength(1)
    expect(reasons[0].key).toBe('soil-unknown')
    expect(reasons[0].text).toContain('Bodenbeschaffenheit')
  })

  it('reports the missing soil type even while the sensor is silent', () => {
    const cluster = makeCluster({
      soilCondition: 'unknown',
      trees: [makeTree('a', { sensor: { status: 'offline', abilities: [moistureAt(40)] } })],
    })

    expect(keysOf(cluster)).toEqual(['soil-unknown', 'sensor-silent'])
  })

  it('does not blame the soil type for a probe at an uncalibrated depth', () => {
    // The EcoDrizzler's 15 cm probe has no KA5 calibration.
    const cluster = makeCluster({
      soilCondition: 'unknown',
      trees: [
        makeTree('a', {
          plantingYear: CURRENT_YEAR,
          sensor: {
            status: 'online',
            abilities: [moistureAt(15), tensionAt(30), tensionAt(60), tensionAt(90)],
          },
        }),
      ],
    })

    expect(keysOf(cluster)).toEqual(['unscorable'])
  })

  it('reports a watermark tree past the monitored growth period', () => {
    const cluster = makeCluster({
      trees: [
        makeTree('a', {
          plantingYear: CURRENT_YEAR - 8,
          sensor: {
            status: 'online',
            abilities: [tensionAt(30), tensionAt(60), tensionAt(90)],
          },
        }),
      ],
    })

    const reasons = unknownStatusReasons(cluster, CURRENT_YEAR)
    expect(reasons).toHaveLength(1)
    expect(reasons[0].key).toBe('beyond-monitoring')
    expect(reasons[0].text).toContain('Anwuchszeitraums')
  })

  it('ignores trees that do have a status', () => {
    const cluster = makeCluster({
      trees: [
        makeTree('a', {
          wateringStatus: 'good',
          sensor: { status: 'online', abilities: [moistureAt(40)] },
        }),
        makeTree('b', { sensor: { status: 'offline', abilities: [moistureAt(40)] } }),
      ],
    })

    const reasons = unknownStatusReasons(cluster, CURRENT_YEAR)
    expect(reasons).toHaveLength(1)
    expect(reasons[0].text).toBe('1 Sensor sendet derzeit keine Daten.')
  })

  it('collects several reasons in a stable order', () => {
    const cluster = makeCluster({
      soilCondition: 'unknown',
      trees: [
        makeTree('a', {
          plantingYear: CURRENT_YEAR - 8,
          sensor: { status: 'online', abilities: [tensionAt(30)] },
        }),
        makeTree('b', { sensor: { status: 'offline', abilities: [moistureAt(40)] } }),
        makeTree('c', { sensor: { status: 'online', abilities: [moistureAt(40)] } }),
      ],
    })

    expect(keysOf(cluster)).toEqual(['soil-unknown', 'sensor-silent', 'beyond-monitoring'])
  })
})
