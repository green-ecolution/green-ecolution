import { beforeEach, describe, expect, it, vi } from 'vitest'
import type { ModuleManifest } from '@/modules/types'
import { validationCatalogKeys } from './validation'

const withCatalog = (key: string, order: number, validation: object) =>
  ({
    key,
    order,
    i18n: { namespaces: [key], load: () => Promise.resolve({ [key]: { validation } }) },
  }) as unknown as ModuleManifest

vi.mock('@/modules/registry', () => ({
  get moduleManifests() {
    return [
      withCatalog('alpha', 1, {
        thing: { name: { empty: 'Alpha: Name fehlt.' } },
        vehicle: { number_plate: { empty: 'Alpha: Kennzeichen fehlt.' } },
      }),
      withCatalog('beta', 2, { thing: { name: { empty: 'Beta: Name fehlt.' } } }),
    ]
  },
}))

beforeEach(() => {
  localStorage.clear()
})

// The test setup has already loaded the i18n modules with the real registry,
// so the catalogs of the mocked modules only arrive in a fresh module graph.
async function translatorWithMockedCatalogs() {
  vi.resetModules()
  const { createI18n } = await import('./index')
  const { translateIssueNow } = await import('./validation')
  await createI18n()
  return translateIssueNow
}

describe('validation catalog lookup', () => {
  it('asks the module of the request first, then the core, then the other modules', () => {
    expect(validationCatalogKeys('thing.name.empty', 'beta')).toEqual([
      'beta:validation.thing.name.empty',
      'validation:thing.name.empty',
      'alpha:validation.thing.name.empty',
    ])
  })

  it('asks the core first without a module', () => {
    expect(validationCatalogKeys('thing.name.empty')).toEqual([
      'validation:thing.name.empty',
      'alpha:validation.thing.name.empty',
      'beta:validation.thing.name.empty',
    ])
  })

  it('takes the text of the module the request belongs to', async () => {
    const translateIssueNow = await translatorWithMockedCatalogs()

    expect(translateIssueNow({ key: 'thing.name.empty', params: {} }, 'beta')).toBe(
      'Beta: Name fehlt.',
    )
    expect(translateIssueNow({ key: 'vehicle.number_plate.empty', params: {} }, 'alpha')).toBe(
      'Alpha: Kennzeichen fehlt.',
    )
  })

  it('prefers the core text when no module is named', async () => {
    const translateIssueNow = await translatorWithMockedCatalogs()

    expect(translateIssueNow({ key: 'vehicle.number_plate.empty', params: {} })).not.toBe(
      'Alpha: Kennzeichen fehlt.',
    )
    expect(translateIssueNow({ key: 'thing.name.empty', params: {} })).toBe('Alpha: Name fehlt.')
  })

  it('falls back to the key for a text nobody has, without throwing', async () => {
    const translateIssueNow = await translatorWithMockedCatalogs()

    expect(translateIssueNow({ key: 'nobody.knows.this', params: {} }, 'alpha')).toBe(
      'nobody.knows.this',
    )
  })
})
