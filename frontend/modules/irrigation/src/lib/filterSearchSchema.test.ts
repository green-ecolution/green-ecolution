import { describe, it, expect } from 'vitest'
import { filterSearchSchema, filtersFromSearch, searchFromFilters } from './filterSearchSchema'

describe('filterSearchSchema', () => {
  it('parses valid filter params', () => {
    const result = filterSearchSchema.parse({
      wateringStatuses: ['good', 'bad'],
      regions: ['0190a8e9-7c4f-7000-8000-000000000000'],
      hasCluster: true,
      plantingYears: [2018, 2020],
    })
    expect(result.wateringStatuses).toEqual(['good', 'bad'])
    expect(result.regions).toEqual(['0190a8e9-7c4f-7000-8000-000000000000'])
    expect(result.hasCluster).toBe(true)
    expect(result.plantingYears).toEqual([2018, 2020])
  })

  it('returns undefined for missing params', () => {
    const result = filterSearchSchema.parse({})
    expect(result.wateringStatuses).toBeUndefined()
    expect(result.regions).toBeUndefined()
    expect(result.hasCluster).toBeUndefined()
    expect(result.plantingYears).toBeUndefined()
  })

  it('drops invalid watering status values instead of throwing', () => {
    const result = filterSearchSchema.parse({ wateringStatuses: ['bogus'] })
    expect(result.wateringStatuses).toBeUndefined()
  })
})

describe('filtersFromSearch / searchFromFilters', () => {
  it('maps search params to dialog draft state', () => {
    expect(
      filtersFromSearch({ wateringStatuses: ['good'], hasCluster: false, plantingYears: [2020] }),
    ).toEqual({
      statusTags: ['good'],
      regionTags: [],
      soilTags: [],
      hasCluster: false,
      plantingYears: [2020],
    })
  })

  it('maps draft state back, omitting empty arrays', () => {
    expect(
      searchFromFilters({
        statusTags: [],
        regionTags: ['r1'],
        soilTags: [],
        hasCluster: undefined,
        plantingYears: [],
      }),
    ).toEqual({
      wateringStatuses: undefined,
      regions: ['r1'],
      hasCluster: undefined,
      plantingYears: undefined,
    })
  })
})
