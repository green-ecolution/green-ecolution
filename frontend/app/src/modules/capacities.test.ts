import { describe, expect, it } from 'vitest'
import type { ModuleListResponse } from '@green-ecolution/backend-client'
import { activeManifestsIn } from './activeModules'
import { capacityKindOf, capacityKindsIn } from './capacities'
import { moduleManifests } from './registry'
import type { ModuleCapacityKind, ModuleManifest } from './types'

const Icon = () => null

const capacity = (kind: string): ModuleCapacityKind =>
  ({ kind, icon: Icon, validate: () => null, texts: {} }) as unknown as ModuleCapacityKind

const manifest = (key: string, kinds: string[] = []) =>
  ({ key, order: 0, capacities: kinds.map(capacity) }) as unknown as ModuleManifest

const response: ModuleListResponse = {
  modules: ['alpha', 'beta'],
  organizations: [
    { organizationId: 'a', active: ['alpha'] },
    { organizationId: 'b', active: ['alpha', 'beta'] },
    { organizationId: 'c', active: [] },
  ],
}

describe('capacity kinds of the registered modules', () => {
  it('every planning module declares its own capacity kind', () => {
    for (const manifest of moduleManifests.filter((m) => m.planning)) {
      const own = manifest.capacities?.map((c) => c.kind) ?? []
      expect(own, manifest.key).toContain(manifest.planning!.capacityKind)
    }
  })

  it('no two modules declare the same kind', () => {
    const kinds = moduleManifests.flatMap((m) => m.capacities?.map((c) => c.kind) ?? [])
    expect(new Set(kinds).size).toBe(kinds.length)
  })
})

describe('activeManifestsIn', () => {
  const manifests = [manifest('alpha'), manifest('beta')]

  it('returns only the modules active in that organization', () => {
    expect(activeManifestsIn(manifests, response, 'a').map((m) => m.key)).toEqual(['alpha'])
    expect(activeManifestsIn(manifests, response, 'b').map((m) => m.key)).toEqual(['alpha', 'beta'])
    expect(activeManifestsIn(manifests, response, 'c')).toEqual([])
  })

  it('knows nothing active in an organization the answer does not list', () => {
    expect(activeManifestsIn(manifests, response, 'unknown')).toEqual([])
  })

  it('drops modules the backend does not register', () => {
    expect(
      activeManifestsIn(manifests, { ...response, modules: ['beta'] }, 'b').map((m) => m.key),
    ).toEqual(['beta'])
  })
})

describe('capacityKindsIn', () => {
  const manifests = [manifest('alpha', ['water']), manifest('beta', ['cargo_volume'])]

  it('offers the kinds of the modules active in the organization', () => {
    expect(capacityKindsIn(manifests, response, 'a').map((c) => c.kind)).toEqual(['water'])
    expect(capacityKindsIn(manifests, response, 'b').map((c) => c.kind)).toEqual([
      'water',
      'cargo_volume',
    ])
  })

  it('offers nothing where no module is active', () => {
    expect(capacityKindsIn(manifests, response, 'c')).toEqual([])
  })

  it('falls back to every reachable organization while the organization is unknown', () => {
    expect(capacityKindsIn(manifests, response, null).map((c) => c.kind)).toEqual([
      'water',
      'cargo_volume',
    ])
  })
})

describe('capacityKindOf', () => {
  const manifests = [manifest('alpha', ['water']), manifest('beta', ['cargo_volume'])]

  it('finds the declaration of a kind in any registered module', () => {
    expect(capacityKindOf('cargo_volume', manifests)?.kind).toBe('cargo_volume')
  })

  it('knows nothing about a kind no module declares', () => {
    expect(capacityKindOf('sand', manifests)).toBeUndefined()
  })
})
