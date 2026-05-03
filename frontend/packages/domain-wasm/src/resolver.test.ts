import { describe, expect, it, vi } from 'vitest'
import type { ValidationIssue } from './types'

vi.mock('../pkg/domain_wasm.js', () => ({
  validateTreeDraft: vi.fn<(_: unknown) => ValidationIssue[]>(),
  validateTreeClusterDraft: vi.fn<(_: unknown) => ValidationIssue[]>(),
  validateVehicleDraft: vi.fn<(_: unknown) => ValidationIssue[]>(),
  validateWateringPlanDraft: vi.fn<(_: unknown) => ValidationIssue[]>(),
}))

import { validateTreeDraft } from '../pkg/domain_wasm.js'
import { treeDraftResolver } from './resolver'

describe('treeDraftResolver', () => {
  it('returns values when validation passes', async () => {
    vi.mocked(validateTreeDraft).mockReturnValueOnce([])
    const values = {
      number: 'FL-001',
      species: 'Quercus',
      plantingYear: 2020,
      latitude: 52.5,
      longitude: 13.4,
    }
    const result = await treeDraftResolver(values, undefined, {} as never)
    expect(result.values).toEqual(values)
    expect(result.errors).toEqual({})
  })

  it('maps issues to RHF FieldErrors keyed by path', async () => {
    vi.mocked(validateTreeDraft).mockReturnValueOnce([
      {
        path: 'species',
        field: 'tree.species',
        key: 'tree.species.empty',
        params: {},
      },
    ])
    const result = await treeDraftResolver(
      { species: '' } as never,
      undefined,
      {} as never,
    )
    expect(result.values).toEqual({})
    expect(result.errors.species).toMatchObject({
      type: 'tree.species.empty',
      message: 'Art ist erforderlich.',
    })
  })

  it('uses i18n key as message fallback when no German string exists', async () => {
    vi.mocked(validateTreeDraft).mockReturnValueOnce([
      {
        path: 'plantingYear',
        field: 'tree.planting_year',
        key: 'tree.planting_year.unknown',
        params: {},
      },
    ])
    const result = await treeDraftResolver({} as never, undefined, {} as never)
    expect(result.errors.plantingYear?.message).toBe('tree.planting_year.unknown')
  })
})
