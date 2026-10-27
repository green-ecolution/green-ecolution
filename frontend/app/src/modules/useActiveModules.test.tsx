import { describe, expect, it } from 'vitest'
import type { ModuleListResponse } from '@green-ecolution/backend-client'
import { activeManifestsFrom, activeModuleKeys } from './activeModules'
import type { ModuleManifest } from './types'

const manifest = (key: string) => ({ key }) as ModuleManifest

const response: ModuleListResponse = {
  modules: ['irrigation', 'waste'],
  organizations: [
    { organizationId: 'a', active: ['irrigation'] },
    { organizationId: 'b', active: [] },
  ],
}

describe('activeModuleKeys', () => {
  it('unites the modules active in any reachable organization', () => {
    expect(activeModuleKeys(response)).toEqual(new Set(['irrigation']))
  })

  it('is empty when no organization has a module', () => {
    expect(activeModuleKeys({ modules: ['waste'], organizations: [] })).toEqual(new Set())
  })
})

describe('activeManifestsFrom', () => {
  const manifests = [manifest('irrigation'), manifest('waste')]

  it('keeps manifests active in at least one visible organization', () => {
    expect(activeManifestsFrom(manifests, response).map((m) => m.key)).toEqual(['irrigation'])
  })

  it('drops manifests of modules the backend does not register', () => {
    const active = activeManifestsFrom(manifests, {
      modules: ['irrigation'],
      organizations: [{ organizationId: 'a', active: ['irrigation', 'waste'] }],
    })
    expect(active.map((m) => m.key)).toEqual(['irrigation'])
  })
})
