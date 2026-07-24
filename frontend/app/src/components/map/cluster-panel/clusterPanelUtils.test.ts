import { describe, it, expect } from 'vitest'
import type { TreeResponse } from '@/api/backendApi'
import type { ClusterMarkerResponse } from '@/api/backendApi'
import {
  sortTreesSensorFirst,
  summarizeTopSpecies,
  filterMarkersByName,
  filterMarkersByStatus,
  latestSensorReading,
} from './clusterPanelUtils'

const tree = (over: Partial<TreeResponse>): TreeResponse => ({
  id: over.id ?? '1',
  createdAt: '',
  updatedAt: '',
  description: '',
  latitude: 0,
  longitude: 0,
  number: over.number ?? '0001',
  organizationId: '',
  plantingYear: 2000,
  sharedWith: [],
  species: over.species ?? 'Stieleiche',
  wateringStatus: 'unknown',
  sensor: over.sensor ?? null,
  ...over,
})

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

describe('filterMarkersByName', () => {
  const m = (name: string): ClusterMarkerResponse => ({ id: name, name }) as ClusterMarkerResponse

  it('returns all markers for an empty term', () => {
    const data = [m('Hafenspitze'), m('Bahnhof')]
    expect(filterMarkersByName(data, '')).toEqual(data)
  })

  it('filters case-insensitively by name substring', () => {
    const data = [m('Hafenspitze'), m('Bahnhof'), m('Hafenstraße')]
    expect(filterMarkersByName(data, 'hafen').map((x) => x.name)).toEqual([
      'Hafenspitze',
      'Hafenstraße',
    ])
  })
})

describe('filterMarkersByStatus', () => {
  const m = (id: string, wateringStatus: string): ClusterMarkerResponse =>
    ({ id, name: id, wateringStatus }) as ClusterMarkerResponse

  const data = [m('a', 'bad'), m('b', 'good'), m('c', 'moderate'), m('d', 'bad')]

  it('returns all markers when no statuses given', () => {
    expect(filterMarkersByStatus(data, undefined)).toEqual(data)
    expect(filterMarkersByStatus(data, [])).toEqual(data)
  })

  it('keeps only markers whose status is selected', () => {
    expect(filterMarkersByStatus(data, ['bad'] as never).map((x) => x.id)).toEqual(['a', 'd'])
    expect(filterMarkersByStatus(data, ['good', 'moderate'] as never).map((x) => x.id)).toEqual([
      'b',
      'c',
    ])
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
