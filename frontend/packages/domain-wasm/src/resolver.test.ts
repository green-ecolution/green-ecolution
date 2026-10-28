import { describe, expect, it, vi } from 'vitest'
import type { TreeForm, ValidationIssue, WasteBinForm } from './types'

vi.mock('../pkg/domain_wasm.js', () => ({
  irrigationValidateTreeDraft: vi.fn<(_: unknown) => ValidationIssue[]>(),
  irrigationValidateTreeClusterDraft: vi.fn<(_: unknown) => ValidationIssue[]>(),
  validateVehicleDraft: vi.fn<(_: unknown) => ValidationIssue[]>(),
  validateWateringPlanDraft: vi.fn<(_: unknown) => ValidationIssue[]>(),
  wasteValidateBinDraft: vi.fn<(_: unknown) => ValidationIssue[]>(),
}))

import { irrigationValidateTreeDraft, wasteValidateBinDraft } from '../pkg/domain_wasm.js'
import { treeDraftResolver, wasteBinDraftResolver } from './resolver'

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

const translate = (key: string) => key

const resolver = treeDraftResolver<TreeForm>(translate)

describe('treeDraftResolver', () => {
  it('returns values when validation passes', async () => {
    vi.mocked(irrigationValidateTreeDraft).mockReturnValueOnce([])
    const result = await resolver(validTreeForm, undefined, {
      fields: {},
      shouldUseNativeValidation: false,
    })
    expect(result.values).toEqual(validTreeForm)
    expect(result.errors).toEqual({})
  })

  it('maps issues to RHF FieldErrors keyed by path', async () => {
    vi.mocked(irrigationValidateTreeDraft).mockReturnValueOnce([
      {
        path: 'species',
        field: 'tree.species',
        key: 'tree.species.empty',
        params: {},
      },
    ])
    const result = await resolver({ ...validTreeForm, species: '' }, undefined, {
      fields: {},
      shouldUseNativeValidation: false,
    })
    expect(result.values).toEqual({})
    expect('species' in result.errors && result.errors.species).toMatchObject({
      type: 'tree.species.empty',
      message: 'tree.species.empty',
    })
  })

  it('uses i18n key as message fallback when no translation exists', async () => {
    vi.mocked(irrigationValidateTreeDraft).mockReturnValueOnce([
      {
        path: 'plantingYear',
        field: 'tree.planting_year',
        key: 'tree.planting_year.unknown',
        params: {},
      },
    ])
    const result = await resolver(validTreeForm, undefined, {
      fields: {},
      shouldUseNativeValidation: false,
    })
    expect('plantingYear' in result.errors && result.errors.plantingYear?.message).toBe(
      'tree.planting_year.unknown',
    )
  })
})

describe('wasteBinDraftResolver', () => {
  const form: WasteBinForm = {
    number: '',
    latitude: 54.78,
    longitude: 9.43,
    volumeLiters: 1100,
    depthCm: 120,
  }
  const binResolver = wasteBinDraftResolver<WasteBinForm>(translate)
  const options = { fields: {}, shouldUseNativeValidation: false }

  it('returns values when validation passes', async () => {
    vi.mocked(wasteValidateBinDraft).mockReturnValueOnce([])
    const result = await binResolver(form, undefined, options)
    expect(result.values).toEqual(form)
    expect(result.errors).toEqual({})
  })

  it('maps issues to field errors keyed by path', async () => {
    vi.mocked(wasteValidateBinDraft).mockReturnValueOnce([
      { path: 'depthCm', field: 'waste_bin.depth', key: 'waste_bin.depth.outOfRange', params: {} },
    ])
    const result = await binResolver({ ...form, depthCm: 0 }, undefined, options)
    expect(result.values).toEqual({})
    expect('depthCm' in result.errors && result.errors.depthCm).toMatchObject({
      type: 'waste_bin.depth.outOfRange',
    })
  })
})
