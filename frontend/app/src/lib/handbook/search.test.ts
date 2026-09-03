import { describe, expect, it } from 'vitest'
import { searchHandbook } from './search'
import type { SearchEntry } from './types'

const entries: SearchEntry[] = [
  {
    slug: 'watering-plans',
    anchor: 'route-festlegen',
    sectionTitle: 'Route festlegen',
    text: 'Erst die Gruppen wählen, dann die Route berechnen lassen.',
  },
  {
    slug: 'sensors',
    anchor: 'zustande',
    sectionTitle: 'Zustände',
    text: 'Ein Sensor ist vorbereitet, online oder offline.',
  },
]

describe('searchHandbook', () => {
  it('returns nothing for a query shorter than two characters', () => {
    expect(searchHandbook(entries, 'r')).toEqual([])
  })

  it('finds a match in the section text regardless of case', () => {
    const hits = searchHandbook(entries, 'ROUTE')

    expect(hits).toHaveLength(1)
    expect(hits[0]).toMatchObject({ slug: 'watering-plans', anchor: 'route-festlegen' })
  })

  it('finds a match in the section title', () => {
    expect(searchHandbook(entries, 'zustände')[0].slug).toBe('sensors')
  })

  it('cuts an excerpt around the match', () => {
    expect(searchHandbook(entries, 'berechnen')[0].excerpt).toContain('berechnen')
  })

  it('ranks a title match above a body match', () => {
    const hits = searchHandbook(entries, 'route')

    expect(hits[0].sectionTitle).toBe('Route festlegen')
  })
})
