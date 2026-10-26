import { describe, expect, it } from 'vitest'
import { probeTarget, probeTargetManifest } from '@/test/targetManifest'
import { sensorTargetKinds, soleTargetKind, targetDescriptor, targetTextKey } from './targets'

const probe = probeTarget('probe', 'probe')
const bin = probeTarget('waste', 'bin')
const manifests = [probeTargetManifest('probe', [probe]), probeTargetManifest('waste', [bin])]

describe('targetDescriptor', () => {
  it('finds the target kind of the module the reference names', () => {
    expect(targetDescriptor(manifests, { module: 'waste', kind: 'bin', id: 'b-1' })).toBe(bin)
  })

  it('does not mix up a kind of the same name in another module', () => {
    expect(targetDescriptor(manifests, { module: 'probe', kind: 'bin', id: 'b-1' })).toBeUndefined()
  })

  it('knows nothing about a missing reference or an unregistered module', () => {
    expect(targetDescriptor(manifests, null)).toBeUndefined()
    expect(targetDescriptor(manifests, undefined)).toBeUndefined()
    expect(targetDescriptor(manifests, { module: 'roads', kind: 'sign', id: 's' })).toBeUndefined()
  })
})

describe('sensorTargetKinds', () => {
  it('lists every target kind together with its module', () => {
    expect(sensorTargetKinds(manifests)).toEqual([
      { module: 'probe', target: probe },
      { module: 'waste', target: bin },
    ])
  })
})

describe('soleTargetKind', () => {
  it('names the kind when only one is on offer', () => {
    expect(soleTargetKind([manifests[0]])).toBe(probe)
  })

  it('names none when the user has to choose', () => {
    expect(soleTargetKind(manifests)).toBeUndefined()
    expect(soleTargetKind([])).toBeUndefined()
  })
})

describe('targetTextKey', () => {
  it('takes the wording the target module gives', () => {
    const worded = probeTarget('probe', 'probe', { texts: { reassignAction: 'irrigation:label' } })
    expect(targetTextKey(worded, 'reassignAction', 'sensor:actions.reassign')).toBe(
      'irrigation:label',
    )
  })

  it('keeps the core text otherwise', () => {
    expect(targetTextKey(probe, 'reassignAction', 'sensor:actions.reassign')).toBe(
      'sensor:actions.reassign',
    )
    expect(targetTextKey(undefined, 'reassignAction', 'sensor:actions.reassign')).toBe(
      'sensor:actions.reassign',
    )
  })
})
