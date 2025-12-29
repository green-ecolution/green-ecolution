import { describe, it, expect } from 'vitest'
import { clusterSchema, clusterSchemaBase } from './treeclusterSchema'
import { SoilCondition } from '@green-ecolution/backend-client'

describe('clusterSchemaBase', () => {
  const validCluster = {
    name: 'Test Cluster',
    address: 'Test Address 123',
    description: 'A test cluster',
    soilCondition: SoilCondition.TreeSoilConditionSandig,
    treeIds: [1, 2, 3],
  }

  it('validates a complete valid cluster object', () => {
    const result = clusterSchemaBase.safeParse(validCluster)
    expect(result.success).toBe(true)
  })

  it('accepts empty name in base schema', () => {
    const result = clusterSchemaBase.safeParse({ ...validCluster, name: '' })
    expect(result.success).toBe(true)
  })

  it('accepts empty address in base schema', () => {
    const result = clusterSchemaBase.safeParse({ ...validCluster, address: '' })
    expect(result.success).toBe(true)
  })

  it('accepts empty description', () => {
    const result = clusterSchemaBase.safeParse({ ...validCluster, description: '' })
    expect(result.success).toBe(true)
  })

  it('accepts all valid SoilCondition values', () => {
    const soilConditions = [
      SoilCondition.TreeSoilConditionSchluffig,
      SoilCondition.TreeSoilConditionSandig,
      SoilCondition.TreeSoilConditionLehmig,
      SoilCondition.TreeSoilConditionTonig,
      SoilCondition.TreeSoilConditionUnknown,
    ]

    soilConditions.forEach((soilCondition) => {
      const result = clusterSchemaBase.safeParse({ ...validCluster, soilCondition })
      expect(result.success).toBe(true)
    })
  })

  it('rejects invalid SoilCondition value', () => {
    const result = clusterSchemaBase.safeParse({ ...validCluster, soilCondition: 'invalid' })
    expect(result.success).toBe(false)
  })

  it('accepts empty treeIds array', () => {
    const result = clusterSchemaBase.safeParse({ ...validCluster, treeIds: [] })
    expect(result.success).toBe(true)
  })

  it('accepts treeIds with valid integers', () => {
    const result = clusterSchemaBase.safeParse({ ...validCluster, treeIds: [1, 2, 3, 100] })
    expect(result.success).toBe(true)
  })

  it('rejects non-integer values in treeIds', () => {
    const result = clusterSchemaBase.safeParse({ ...validCluster, treeIds: [1.5, 2.7] })
    expect(result.success).toBe(false)
  })
})

describe('clusterSchema (extended)', () => {
  const validCluster = {
    name: 'Test Cluster',
    address: 'Test Address 123',
    description: '',
    soilCondition: SoilCondition.TreeSoilConditionSandig,
    treeIds: [],
  }

  it('requires non-empty name', () => {
    const result = clusterSchema.safeParse({ ...validCluster, name: '' })
    expect(result.success).toBe(false)
    if (!result.success) {
      expect(result.error.issues[0].message).toContain('Name')
    }
  })

  it('requires non-empty address', () => {
    const result = clusterSchema.safeParse({ ...validCluster, address: '' })
    expect(result.success).toBe(false)
    if (!result.success) {
      expect(result.error.issues[0].message).toContain('Adresse')
    }
  })

  it('allows empty description', () => {
    const result = clusterSchema.safeParse({ ...validCluster, description: '' })
    expect(result.success).toBe(true)
  })

  it('allows description with content', () => {
    const result = clusterSchema.safeParse({
      ...validCluster,
      description: 'This is a detailed description',
    })
    expect(result.success).toBe(true)
  })

  it('allows empty treeIds array', () => {
    const result = clusterSchema.safeParse({ ...validCluster, treeIds: [] })
    expect(result.success).toBe(true)
  })

  it('validates complete cluster with all fields', () => {
    const result = clusterSchema.safeParse({
      name: 'Complete Cluster',
      address: 'Full Address 456',
      description: 'A complete cluster description',
      soilCondition: SoilCondition.TreeSoilConditionLehmig,
      treeIds: [1, 2, 3],
    })
    expect(result.success).toBe(true)
  })
})
