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
