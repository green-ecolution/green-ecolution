import { describe, it, expect } from 'vitest'
import {
  wateringPlanSchemaBase,
  wateringPlanSchema,
  wateringPlanFinishedSchema,
  wateringPlanCancelSchema,
} from './wateringPlanSchema'
import { WateringPlanStatus } from '@green-ecolution/backend-client'

describe('wateringPlanSchemaBase', () => {
  const validWateringPlan = {
    date: new Date(),
    status: WateringPlanStatus.WateringPlanStatusPlanned,
    transporterId: 1,
    trailerId: 2,
    driverIds: ['550e8400-e29b-41d4-a716-446655440000'],
    clusterIds: [1, 2, 3],
    description: 'Test watering plan',
  }

  it('validates a complete valid watering plan object', () => {
    const result = wateringPlanSchemaBase.safeParse(validWateringPlan)
    expect(result.success).toBe(true)
  })

  it('accepts all valid WateringPlanStatus values', () => {
    const statuses = [
      WateringPlanStatus.WateringPlanStatusPlanned,
      WateringPlanStatus.WateringPlanStatusActive,
      WateringPlanStatus.WateringPlanStatusCanceled,
      WateringPlanStatus.WateringPlanStatusFinished,
      WateringPlanStatus.WateringPlanStatusNotCompeted,
      WateringPlanStatus.WateringPlanStatusUnknown,
    ]

    statuses.forEach((status) => {
      const result = wateringPlanSchemaBase.safeParse({ ...validWateringPlan, status })
      expect(result.success).toBe(true)
    })
  })

  it('rejects invalid status value', () => {
    const result = wateringPlanSchemaBase.safeParse({ ...validWateringPlan, status: 'invalid' })
    expect(result.success).toBe(false)
  })

  it('requires positive transporterId', () => {
    const result = wateringPlanSchemaBase.safeParse({ ...validWateringPlan, transporterId: 0 })
    expect(result.success).toBe(false)
  })

  it('accepts optional trailerId', () => {
    const withoutTrailer = { ...validWateringPlan }
    delete (withoutTrailer as Partial<typeof validWateringPlan>).trailerId
    const result = wateringPlanSchemaBase.safeParse(withoutTrailer)
    expect(result.success).toBe(true)
  })

  it('requires valid UUID for driverIds', () => {
    const result = wateringPlanSchemaBase.safeParse({
      ...validWateringPlan,
      driverIds: ['not-a-uuid'],
    })
    expect(result.success).toBe(false)
  })

  it('accepts empty driverIds array in base schema', () => {
    const result = wateringPlanSchemaBase.safeParse({ ...validWateringPlan, driverIds: [] })
    expect(result.success).toBe(true)
  })

  it('accepts empty clusterIds array in base schema', () => {
    const result = wateringPlanSchemaBase.safeParse({ ...validWateringPlan, clusterIds: [] })
    expect(result.success).toBe(true)
  })

  it('coerces string date to Date object', () => {
    const result = wateringPlanSchemaBase.safeParse({
      ...validWateringPlan,
      date: '2025-12-31',
    })
    expect(result.success).toBe(true)
    if (result.success) {
      expect(result.data.date instanceof Date).toBe(true)
    }
  })

  it('coerces string transporterId to number', () => {
    const result = wateringPlanSchemaBase.safeParse({
      ...validWateringPlan,
      transporterId: '5',
    })
    expect(result.success).toBe(true)
    if (result.success) {
      expect(result.data.transporterId).toBe(5)
    }
  })
})

describe('wateringPlanSchema (extended)', () => {
  const futureDate = new Date()
  futureDate.setDate(futureDate.getDate() + 7)

  const validWateringPlan = {
    date: futureDate,
    status: WateringPlanStatus.WateringPlanStatusPlanned,
    transporterId: 1,
    driverIds: ['550e8400-e29b-41d4-a716-446655440000'],
    clusterIds: [1],
    description: '',
  }

  it('requires date in the future', () => {
    const pastDate = new Date()
    pastDate.setDate(pastDate.getDate() - 1)

    const result = wateringPlanSchema.safeParse({ ...validWateringPlan, date: pastDate })
    expect(result.success).toBe(false)
    if (!result.success) {
      expect(result.error.issues[0].message).toContain('Zukunft')
    }
  })

  it('requires at least one driver', () => {
    const result = wateringPlanSchema.safeParse({ ...validWateringPlan, driverIds: [] })
    expect(result.success).toBe(false)
    if (!result.success) {
      expect(result.error.issues[0].message).toContain('Mitwarbeiter')
    }
  })

  it('requires at least one cluster', () => {
    const result = wateringPlanSchema.safeParse({ ...validWateringPlan, clusterIds: [] })
    expect(result.success).toBe(false)
  })

  it('accepts multiple drivers', () => {
    const result = wateringPlanSchema.safeParse({
      ...validWateringPlan,
      driverIds: ['550e8400-e29b-41d4-a716-446655440000', '550e8400-e29b-41d4-a716-446655440001'],
    })
    expect(result.success).toBe(true)
  })

  it('accepts multiple clusters', () => {
    const result = wateringPlanSchema.safeParse({
      ...validWateringPlan,
      clusterIds: [1, 2, 3, 4, 5],
    })
    expect(result.success).toBe(true)
  })
})

describe('wateringPlanFinishedSchema', () => {
  it('validates valid evaluation array', () => {
    const result = wateringPlanFinishedSchema.safeParse({
      evaluation: [
        { consumedWater: 100, treeClusterId: 1, wateringPlanId: 1 },
        { consumedWater: 200, treeClusterId: 2, wateringPlanId: 1 },
      ],
    })
    expect(result.success).toBe(true)
  })

  it('requires positive consumedWater', () => {
    const result = wateringPlanFinishedSchema.safeParse({
      evaluation: [{ consumedWater: 0, treeClusterId: 1, wateringPlanId: 1 }],
    })
    expect(result.success).toBe(false)
  })

  it('accepts empty evaluation array', () => {
    const result = wateringPlanFinishedSchema.safeParse({
      evaluation: [],
    })
    expect(result.success).toBe(true)
  })
})

describe('wateringPlanCancelSchema', () => {
  it('requires non-empty cancellationNote', () => {
    const result = wateringPlanCancelSchema.safeParse({
      cancellationNote: '',
    })
    expect(result.success).toBe(false)
  })

  it('accepts valid cancellationNote', () => {
    const result = wateringPlanCancelSchema.safeParse({
      cancellationNote: 'Weather conditions too bad',
    })
    expect(result.success).toBe(true)
  })
})
