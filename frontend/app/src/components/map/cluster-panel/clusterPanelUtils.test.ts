import { describe, it, expect } from 'vitest'
import type { TreeResponse } from '@/api/backendApi'
import { summarizeTopSpecies } from './clusterPanelUtils'

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
