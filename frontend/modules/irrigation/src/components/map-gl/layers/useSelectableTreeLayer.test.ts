import { describe, it, expect } from 'vitest'
import type { TreeMarkerResponse } from '@green-ecolution/backend-client'
import { toFC } from './useSelectableTreeLayer'

const marker = (id: string, organizationId: string) =>
  ({
    id,
    latitude: 54.79,
    longitude: 9.44,
    wateringStatus: 'good',
    number: id,
    hasSensor: false,
    organizationId,
  }) as TreeMarkerResponse

const trees = [marker('own', 'tbz'), marker('foreign', 'extern')]

describe('toFC', () => {
  it('marks trees of other organizations as unselectable but keeps them', () => {
    const features = toFC(trees, new Set(), 'tbz').features

    expect(features).toHaveLength(2)
    expect(features[0].properties).toMatchObject({ id: 'own', selectable: true })
    expect(features[1].properties).toMatchObject({ id: 'foreign', selectable: false })
  })

  it('carries the organization so a click can name it', () => {
    const features = toFC(trees, new Set(), 'tbz').features

    expect(features[1].properties?.organizationId).toBe('extern')
  })

  it('makes everything selectable when no organization is set', () => {
    const features = toFC(trees, new Set(), undefined).features

    expect(features.every((f) => f.properties?.selectable)).toBe(true)
  })

  it('keeps the selected flag independent of selectability', () => {
    const features = toFC(trees, new Set(['own']), 'tbz').features

    expect(features[0].properties?.selected).toBe(true)
    expect(features[1].properties?.selected).toBe(false)
  })
})
