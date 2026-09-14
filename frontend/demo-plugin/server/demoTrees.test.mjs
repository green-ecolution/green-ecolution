import { describe, expect, it } from 'vitest'
import {
  DELETED_EXTERNAL_ID,
  MODIFIED_EXTERNAL_ID,
  MODIFIED_SPECIES,
  importBatch,
  modifiedBatch,
} from './demoTrees.mjs'

describe('importBatch', () => {
  it('carries six trees with the full ingest field set', () => {
    const { items } = importBatch()

    expect(items).toHaveLength(6)
    for (const item of items) {
      expect(Object.keys(item).sort()).toEqual([
        'additional_info',
        'description',
        'external_id',
        'latitude',
        'longitude',
        'number',
        'planting_year',
        'species',
      ])
      expect(item.planting_year).toBeGreaterThanOrEqual(1900)
      expect(item.latitude).toBeGreaterThan(54)
      expect(item.longitude).toBeGreaterThan(9)
    }
  })

  it('uses stable external ids so a second import reports unchanged', () => {
    expect(importBatch().items.map((i) => i.external_id)).toEqual(
      importBatch().items.map((i) => i.external_id),
    )
    expect(new Set(importBatch().items.map((i) => i.external_id)).size).toBe(6)
  })

  it('contains the trees the modify and delete actions address', () => {
    const ids = importBatch().items.map((i) => i.external_id)

    expect(ids).toContain(MODIFIED_EXTERNAL_ID)
    expect(ids).toContain(DELETED_EXTERNAL_ID)
  })
})

describe('modifiedBatch', () => {
  it('differs from the imported tree in exactly one field', () => {
    const original = importBatch().items.find((i) => i.external_id === MODIFIED_EXTERNAL_ID)
    const { items } = modifiedBatch()

    expect(items).toHaveLength(1)
    const changed = Object.keys(items[0]).filter(
      (key) => JSON.stringify(items[0][key]) !== JSON.stringify(original[key]),
    )
    expect(changed).toEqual(['species'])
    expect(items[0].species).toBe(MODIFIED_SPECIES)
  })
})
