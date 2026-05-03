import { describe, expect, it, vi } from 'vitest'
import type { TreeForm, ValidationIssue } from './types'

vi.mock('../pkg/domain_wasm.js', () => ({
  validateTreeDraft: vi.fn<(_: unknown) => ValidationIssue[]>(),
  validateTreeClusterDraft: vi.fn<(_: unknown) => ValidationIssue[]>(),
  validateVehicleDraft: vi.fn<(_: unknown) => ValidationIssue[]>(),
  validateWateringPlanDraft: vi.fn<(_: unknown) => ValidationIssue[]>(),
}))

import { validateTreeDraft } from '../pkg/domain_wasm.js'
import { treeDraftResolver } from './resolver'

const validTreeForm: TreeForm = {
  number: 'FL-001',
  species: 'Quercus',
  plantingYear: 2020,
  latitude: 52.5,
  longitude: 13.4,
  description: '',
  treeClusterId: -1,
  sensorId: '-1',
}

describe('treeDraftResolver', () => {
  it('returns values when validation passes', async () => {
    vi.mocked(validateTreeDraft).mockReturnValueOnce([])
    const result = await treeDraftResolver(validTreeForm, undefined, {} as never)
    expect(result.values).toEqual(validTreeForm)
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
      { ...validTreeForm, species: '' },
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
    const result = await treeDraftResolver(validTreeForm, undefined, {} as never)
    expect(result.errors.plantingYear?.message).toBe('tree.planting_year.unknown')
  })
})
