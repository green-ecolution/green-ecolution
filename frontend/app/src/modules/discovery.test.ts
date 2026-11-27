/// <reference types="node" />
import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from 'node:fs'
import os from 'node:os'
import path from 'node:path'
import { afterEach, beforeEach, describe, expect, it } from 'vitest'
import { discoverModules } from '../../modules.config'
import { moduleRouteNodes } from './buildConfig'

describe('module discovery', () => {
  let dir: string

  beforeEach(() => {
    dir = mkdtempSync(path.join(os.tmpdir(), 'modules-'))
  })
  afterEach(() => {
    rmSync(dir, { recursive: true, force: true })
  })

  const file = (relative: string) => {
    const full = path.join(dir, relative)
    mkdirSync(path.dirname(full), { recursive: true })
    writeFileSync(full, '')
  }

  it('finds the directories that hold src/manifest.tsx, sorted by name', () => {
    file('b/src/manifest.tsx')
    file('b/src/routes/_protected/b/index.tsx')
    file('a/src/manifest.tsx')
    file('a/src/routes/_protected/a/index.tsx')

    expect(discoverModules(dir)).toEqual([
      { key: 'a', root: path.join(dir, 'a'), routesDirectory: 'src/routes' },
      { key: 'b', root: path.join(dir, 'b'), routesDirectory: 'src/routes' },
    ])
  })

  it('orders by code point, the same on every machine', () => {
    file('a/src/manifest.tsx')
    file('B/src/manifest.tsx')

    expect(discoverModules(dir).map((m) => m.key)).toEqual(['B', 'a'])
  })

  it('skips a directory without a manifest', () => {
    file('half/src/index.ts')
    file('README.md')

    expect(discoverModules(dir)).toEqual([])
  })

  it('leaves routesDirectory unset for a module without routes', () => {
    file('a/src/manifest.tsx')

    expect(discoverModules(dir)).toEqual([{ key: 'a', root: path.join(dir, 'a') }])
  })

  it('builds no route node for a module without routes', () => {
    expect(moduleRouteNodes('/app/src/routes', [{ key: 'a', root: '/modules/a' }])).toHaveLength(1)
  })
})
