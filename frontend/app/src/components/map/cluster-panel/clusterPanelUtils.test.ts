import { describe, it, expect } from 'vitest'
import type { TreeResponse } from '@/api/backendApi'
import { sortTreesSensorFirst, summarizeTopSpecies, latestSensorReading } from './clusterPanelUtils'

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

describe('sortTreesSensorFirst', () => {
  it('puts trees with a sensor before trees without, keeping relative order', () => {
    const a = tree({ id: 'a', sensor: null })
    const b = tree({ id: 'b', sensor: { id: 's1' } as never })
    const c = tree({ id: 'c', sensor: null })
    const d = tree({ id: 'd', sensor: { id: 's2' } as never })

    const result = sortTreesSensorFirst([a, b, c, d]).map((t) => t.id)

    expect(result).toEqual(['b', 'd', 'a', 'c'])
  })

  it('does not mutate the input array', () => {
    const input = [tree({ id: 'a', sensor: null }), tree({ id: 'b', sensor: {} as never })]
    const copy = [...input]
    sortTreesSensorFirst(input)
    expect(input).toEqual(copy)
  })
})

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
