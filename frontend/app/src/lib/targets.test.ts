import { describe, expect, it } from 'vitest'
import { probeTarget, probeTargetManifest } from '@/test/targetManifest'
import { sensorTargetKinds, targetDescriptor } from './targets'

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
