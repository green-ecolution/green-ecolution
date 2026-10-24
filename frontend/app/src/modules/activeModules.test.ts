import { describe, expect, it } from 'vitest'
import type { ModuleListResponse } from '@green-ecolution/backend-client'
import { activeManifests, activeModuleKeys } from './activeModules'

const response: ModuleListResponse = {
  modules: ['irrigation', 'waste'],
  organizations: [
    { organizationId: 'a', active: ['irrigation'] },
    { organizationId: 'b', active: ['irrigation', 'waste'] },
    { organizationId: 'c', active: [] },
  ],
}

describe('activeModuleKeys', () => {
  it('unites the modules active in any reachable organization', () => {
    expect(activeModuleKeys(response)).toEqual(new Set(['irrigation', 'waste']))
  })

  it('is empty when no organization has a module', () => {
    expect(activeModuleKeys({ modules: ['waste'], organizations: [] })).toEqual(new Set())
  })
})

describe('activeManifests', () => {
  it('keeps only manifests whose module is active', () => {
    const manifests = [{ key: 'irrigation' }, { key: 'waste' }]
    expect(activeManifests(manifests, new Set(['waste']))).toEqual([{ key: 'waste' }])
  })
})
