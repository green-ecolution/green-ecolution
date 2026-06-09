import { describe, it, expect } from 'vitest'
import type { TreeResponse } from '@/api/backendApi'
import type { ClusterMarkerResponse } from '@/api/backendApi'
import { sortTreesSensorFirst, summarizeTopSpecies, filterMarkersByName } from './clusterPanelUtils'

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
