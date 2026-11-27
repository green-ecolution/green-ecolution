import { describe, expect, it } from 'vitest'
import { MODULE_BUILDS } from '../../modules.config'
import { moduleManifests, sortManifests } from './registry'
import type { ModuleManifest } from './types'

const fake = (key: string, order: number) => ({ key, order }) as ModuleManifest

describe('module registry', () => {
  it('sorts by order', () => {
    expect(sortManifests([fake('b', 20), fake('a', 10)]).map((m) => m.key)).toEqual(['a', 'b'])
  })

  it('breaks a tie by key', () => {
    expect(sortManifests([fake('b', 10), fake('a', 10)]).map((m) => m.key)).toEqual(['a', 'b'])
  })

  it('holds one manifest per module directory, keyed like the directory', () => {
    expect(moduleManifests.map((m) => m.key).sort()).toEqual(MODULE_BUILDS.map((b) => b.key).sort())
  })

  it('is sorted', () => {
    expect(moduleManifests).toEqual(sortManifests(moduleManifests))
  })
})
