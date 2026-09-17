import { describe, it, expect } from 'vitest'
import type { z } from 'zod'
import { Route } from './index'

const parseSearch = (input: Record<string, unknown>) =>
  (Route.options.validateSearch as unknown as z.ZodSchema<{ page: number }>).parse(input)

const loaderDeps = Route.options.loaderDeps as unknown as (opts: {
  search: Record<string, unknown>
}) => Record<string, unknown>

describe('/trees route pagination (GECO-129)', () => {
  it('parses page from search params', () => {
    expect(parseSearch({ page: 2 }).page).toBe(2)
  })

  it('falls back to page 1 when page is missing or invalid', () => {
    expect(parseSearch({}).page).toBe(1)
    expect(parseSearch({ page: 'invalid' }).page).toBe(1)
  })

  it('passes page from search params through loaderDeps', () => {
    expect(loaderDeps({ search: { page: 3 } }).page).toBe(3)
  })
})

describe('/trees route filters (GECO-133)', () => {
  it('parses filter params from search', () => {
    const result = parseSearch({
      wateringStatuses: ['good'],
      hasCluster: true,
      plantingYears: [2020],
    }) as Record<string, unknown>
    expect(result.wateringStatuses).toEqual(['good'])
    expect(result.hasCluster).toBe(true)
    expect(result.plantingYears).toEqual([2020])
  })

  it('drops invalid watering statuses instead of throwing', () => {
    const result = parseSearch({ wateringStatuses: ['bogus'], page: 1 }) as Record<string, unknown>
    expect(result.wateringStatuses).toBeUndefined()
  })

  it('passes filters through loaderDeps', () => {
    const deps = loaderDeps({
      search: { page: 1, wateringStatuses: ['good'], hasCluster: false, plantingYears: [2018] },
    })
    expect(deps.wateringStatuses).toEqual(['good'])
    expect(deps.hasCluster).toBe(false)
    expect(deps.plantingYears).toEqual([2018])
  })
})

describe('/trees route search, sort and new filters', () => {
  it('parses the query string', () => {
    const result = parseSearch({ q: 'Quercus' }) as Record<string, unknown>
    expect(result.q).toBe('Quercus')
  })

  it('parses sort and order', () => {
    const result = parseSearch({ sort: 'species', order: 'desc' }) as Record<string, unknown>
    expect(result.sort).toBe('species')
    expect(result.order).toBe('desc')
  })

  it('drops an unknown sort field instead of throwing', () => {
    const result = parseSearch({ sort: 'height' }) as Record<string, unknown>
    expect(result.sort).toBeUndefined()
  })

  it('drops an unknown order instead of throwing', () => {
    const result = parseSearch({ order: 'sideways' }) as Record<string, unknown>
    expect(result.order).toBeUndefined()
  })

  it('parses the cluster and sensor filters', () => {
    const result = parseSearch({
      clusterIds: ['0190a8e9-7c4f-7000-8000-000000000000'],
      hasSensor: false,
    }) as Record<string, unknown>
    expect(result.clusterIds).toEqual(['0190a8e9-7c4f-7000-8000-000000000000'])
    expect(result.hasSensor).toBe(false)
  })

  it('passes every new key through loaderDeps', () => {
    const deps = loaderDeps({
      search: {
        page: 1,
        q: 'Quercus',
        sort: 'species',
        order: 'desc',
        clusterIds: ['0190a8e9-7c4f-7000-8000-000000000000'],
        hasSensor: true,
      },
    })
    expect(deps.q).toBe('Quercus')
    expect(deps.sort).toBe('species')
    expect(deps.order).toBe('desc')
    expect(deps.clusterIds).toEqual(['0190a8e9-7c4f-7000-8000-000000000000'])
    expect(deps.hasSensor).toBe(true)
  })
})
