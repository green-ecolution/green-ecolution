import { describe, expect, it, vi } from 'vitest'
import { draftResolver } from './resolver'
import type { ValidationIssue } from './types'

interface ThingForm {
  name: string
  builtAt: Date
  parts: { checkedAt: Date }[]
}

const form: ThingForm = {
  name: 'Bench',
  builtAt: new Date('2026-05-01T08:00:00.000Z'),
  parts: [{ checkedAt: new Date('2026-06-01T08:00:00.000Z') }],
}

const options = { fields: {}, shouldUseNativeValidation: false }
const issue = (overrides: Partial<ValidationIssue> = {}): ValidationIssue => ({
  path: 'name',
  field: 'thing.name',
  key: 'thing.name.empty',
  params: {},
  ...overrides,
})

describe('draftResolver', () => {
  it('returns the values when the validator finds nothing', async () => {
    const resolver = draftResolver<ThingForm>(
      () => [],
      (key) => key,
    )

    const result = await resolver(form, undefined, options)

    expect(result.values).toEqual(form)
    expect(result.errors).toEqual({})
  })

  it('builds a resolver from any validator', async () => {
    const validate = vi.fn(() => [issue()])
    const resolver = draftResolver<ThingForm>(validate, (key) => `text:${key}`)

    const result = await resolver({ ...form, name: '' }, undefined, options)

    expect(result.values).toEqual({})
    expect(result.errors.name).toEqual({
      type: 'thing.name.empty',
      message: 'text:thing.name.empty',
    })
  })

  it('nests a dotted issue path so React Hook Form finds the error', async () => {
    const resolver = draftResolver<ThingForm>(
      () => [issue({ path: 'capacities.water', key: 'capacity.amount.outOfRange' })],
      (key) => `text:${key}`,
    )

    const result = await resolver(form, undefined, options)
    const errors = result.errors as Record<string, Record<string, { message?: string }>>

    expect(errors.capacities.water.message).toBe('text:capacity.amount.outOfRange')
    expect(result.errors).not.toHaveProperty(['capacities.water'])
  })

  it('shows the key when the translator has no text', async () => {
    const resolver = draftResolver<ThingForm>(
      () => [issue()],
      () => '',
    )

    const result = await resolver(form, undefined, options)

    expect(result.errors.name?.message).toBe('thing.name.empty')
  })

  it('passes dates to the validator as ISO strings, however deep they sit', async () => {
    const validate = vi.fn((_: unknown) => [])
    const resolver = draftResolver<ThingForm>(validate, (key) => key)

    await resolver(form, undefined, options)

    expect(validate).toHaveBeenCalledWith({
      name: 'Bench',
      builtAt: '2026-05-01T08:00:00.000Z',
      parts: [{ checkedAt: '2026-06-01T08:00:00.000Z' }],
    })
  })
})
