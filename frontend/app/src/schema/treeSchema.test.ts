import { describe, it, expect } from 'vitest'
import { treeSchema, treeSchemaBase } from './treeSchema'

describe('treeSchemaBase', () => {
  const validTree = {
    latitude: 53.5511,
    longitude: 9.9937,
    number: 'T-001',
    species: 'Oak',
    plantingYear: 2023,
    treeClusterId: 1,
    sensorId: 'sensor-1',
    description: 'A beautiful oak tree',
  }

  it('validates a complete valid tree object', () => {
    const result = treeSchemaBase.safeParse(validTree)
    expect(result.success).toBe(true)
  })

  it('rejects latitude greater than 90', () => {
    const result = treeSchemaBase.safeParse({ ...validTree, latitude: 100 })
    expect(result.success).toBe(false)
  })

  it('rejects latitude less than -90', () => {
    const result = treeSchemaBase.safeParse({ ...validTree, latitude: -100 })
    expect(result.success).toBe(false)
  })

  it('rejects longitude greater than 180', () => {
    const result = treeSchemaBase.safeParse({ ...validTree, longitude: 200 })
    expect(result.success).toBe(false)
  })

  it('rejects longitude less than -180', () => {
    const result = treeSchemaBase.safeParse({ ...validTree, longitude: -200 })
    expect(result.success).toBe(false)
  })

  it('rejects planting year before 2020', () => {
    const result = treeSchemaBase.safeParse({ ...validTree, plantingYear: 2019 })
    expect(result.success).toBe(false)
    if (!result.success) {
      expect(result.error.issues[0].message).toContain('2020')
    }
  })

  it('rejects future planting years', () => {
    const futureYear = new Date().getFullYear() + 1
    const result = treeSchemaBase.safeParse({ ...validTree, plantingYear: futureYear })
    expect(result.success).toBe(false)
  })

  it('accepts current year as planting year', () => {
    const currentYear = new Date().getFullYear()
    const result = treeSchemaBase.safeParse({ ...validTree, plantingYear: currentYear })
    expect(result.success).toBe(true)
  })
})

describe('treeSchema (extended)', () => {
  const validTree = {
    latitude: 53.5511,
    longitude: 9.9937,
    number: 'T-001',
    species: 'Oak',
    plantingYear: 2023,
    treeClusterId: 1,
    sensorId: 'sensor-1',
    description: '',
  }

  it('requires non-empty number', () => {
    const result = treeSchema.safeParse({ ...validTree, number: '' })
    expect(result.success).toBe(false)
    if (!result.success) {
      expect(result.error.issues[0].message).toContain('Baumnummer')
    }
  })

  it('requires non-empty species', () => {
    const result = treeSchema.safeParse({ ...validTree, species: '' })
    expect(result.success).toBe(false)
    if (!result.success) {
      expect(result.error.issues[0].message).toContain('Art')
    }
  })

  it('allows -1 for treeClusterId (no cluster selected)', () => {
    const result = treeSchema.safeParse({ ...validTree, treeClusterId: -1 })
    expect(result.success).toBe(true)
  })

  it('allows positive number for treeClusterId', () => {
    const result = treeSchema.safeParse({ ...validTree, treeClusterId: 5 })
    expect(result.success).toBe(true)
  })

  it('allows "-1" for sensorId (no sensor selected)', () => {
    const result = treeSchema.safeParse({ ...validTree, sensorId: '-1' })
    expect(result.success).toBe(true)
  })

  it('allows string sensor id', () => {
    const result = treeSchema.safeParse({ ...validTree, sensorId: 'sensor-123' })
    expect(result.success).toBe(true)
  })

  it('allows empty description', () => {
    const result = treeSchema.safeParse({ ...validTree, description: '' })
    expect(result.success).toBe(true)
  })

  it('allows description with content', () => {
    const result = treeSchema.safeParse({
      ...validTree,
      description: 'This is a test description',
    })
    expect(result.success).toBe(true)
  })
})
