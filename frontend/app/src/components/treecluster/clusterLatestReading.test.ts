import { describe, it, expect } from 'vitest'
import { latestClusterReading } from './clusterLatestReading'
import type { Tree } from '@/api/backendApi'

interface TreeOverrides {
  id?: string
  sensor?: { id: string; latestData: NonNullable<Tree['sensor']>['latestData'] } | null
}

function makeTree(overrides: TreeOverrides = {}): Tree {
  return {
    id: '1',
    species: 'Ahorn',
    number: 'B-001',
    sensor: null,
    ...overrides,
  } as Tree
}

function makeSensorData(data: object, updatedAt: string) {
  return {
    id: 'reading-1',
    createdAt: updatedAt,
    updatedAt,
    data,
  }
}

describe('latestClusterReading', () => {
  it('reads the temperature from an EcoDrizzler-shaped payload', () => {
    const trees = [
      makeTree({
        sensor: {
          id: 'sensor-1',
          latestData: makeSensorData({ temperature: 21.4 }, '2024-01-01T12:00:00Z'),
        },
      }),
    ]

    expect(latestClusterReading(trees)).toEqual({
      temperature: 21.4,
      measuredAt: new Date('2024-01-01T12:00:00Z'),
    })
  })

  it('reads the temperature from a generic readings-array payload', () => {
    const trees = [
      makeTree({
        sensor: {
          id: 'sensor-1',
          latestData: makeSensorData(
            { readings: [{ ability: 'temperature', depth: 15, value: 19.2 }] },
            '2024-01-01T12:00:00Z',
          ),
        },
      }),
    ]

    expect(latestClusterReading(trees)).toEqual({
      temperature: 19.2,
      measuredAt: new Date('2024-01-01T12:00:00Z'),
    })
  })

  it('uses the newer reading when multiple sensors report data', () => {
    const trees = [
      makeTree({
        id: '1',
        sensor: {
          id: 'sensor-1',
          latestData: makeSensorData({ temperature: 10 }, '2024-01-01T08:00:00Z'),
        },
      }),
      makeTree({
        id: '2',
        sensor: {
          id: 'sensor-2',
          latestData: makeSensorData({ temperature: 15.5 }, '2024-01-02T08:00:00Z'),
        },
      }),
    ]

    expect(latestClusterReading(trees)).toEqual({
      temperature: 15.5,
      measuredAt: new Date('2024-01-02T08:00:00Z'),
    })
  })

  it('returns null values when no tree has a sensor', () => {
    const trees = [makeTree({ sensor: null }), makeTree({ id: '2', sensor: null })]

    expect(latestClusterReading(trees)).toEqual({
      temperature: null,
      measuredAt: null,
    })
  })
})
