import { describe, it, expect } from 'vitest'
import type { TreeResponse } from '@/api/backendApi'
import { summarizeTopSpecies, latestSensorReading } from './clusterPanelUtils'

const tree = (over: Partial<TreeResponse>): TreeResponse =>
  ({
    id: over.id ?? '1',
    createdAt: '',
    updatedAt: '',
    description: '',
    latitude: 0,
    longitude: 0,
    number: over.number ?? '0001',
    plantingYear: 2000,
    species: over.species ?? 'Stieleiche',
    wateringStatus: 'unknown',
    sensor: over.sensor ?? null,
    ...over,
  }) as TreeResponse

describe('summarizeTopSpecies', () => {
  it('returns the two most frequent species, comma-joined', () => {
    const trees = [
      tree({ species: 'Stieleiche' }),
      tree({ species: 'Stieleiche' }),
      tree({ species: 'Spitzahorn' }),
      tree({ species: 'Linde' }),
    ]
    expect(summarizeTopSpecies(trees)).toBe('Stieleiche, Spitzahorn')
  })

  it('returns an empty string for no trees', () => {
    expect(summarizeTopSpecies([])).toBe('')
  })
})

describe('latestSensorReading', () => {
  const treeWith = (createdAt: string | null): TreeResponse =>
    ({
      sensor: createdAt ? { latestData: { createdAt, data: {} } } : null,
    }) as unknown as TreeResponse

  it('returns the most recent reading across trees', () => {
    const trees = [
      treeWith('2026-06-01T10:00:00Z'),
      treeWith(null),
      treeWith('2026-06-09T08:00:00Z'),
      treeWith('2026-06-05T12:00:00Z'),
    ]
    expect(latestSensorReading(trees)?.createdAt).toBe('2026-06-09T08:00:00Z')
  })

  it('returns undefined when no tree has a sensor reading', () => {
    expect(latestSensorReading([treeWith(null), treeWith(null)])).toBeUndefined()
  })
})
