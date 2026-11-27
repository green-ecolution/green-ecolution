import {
  existsSync,
  mkdirSync,
  mkdtempSync,
  readdirSync,
  readFileSync,
  rmSync,
  statSync,
  writeFileSync,
} from 'node:fs'
import os from 'node:os'
import path from 'node:path'
import { afterEach, beforeEach, describe, expect, it } from 'vitest'
import { MODULE_BUILDS } from '../../modules.config'
import type { ModuleBuild } from './buildConfig'
import { moduleManifests } from './registry'
import type { ModuleManifest } from './types'

const SRC = path.resolve(import.meta.dirname, '..')
const MODULE_PACKAGES = path.resolve(SRC, '../../modules')
// Fixtures and MSW handlers are test code like the *.test files: they stand in
// for a backend that has a module registered.
const TEST_SUPPORT = path.join(SRC, 'test')
const COMPOSITION_ROOTS = new Set(['modules/registry.ts', 'routeTree.gen.ts'])
const ACTIONS = 'read|create|update|delete'

/**
 * URL prefixes of the routes a module adds. Its routes directory mirrors the
 * app's, so a path belongs to the module from the first entry the app's
 * directory does not have.
 */
function ownedRoutePrefixes(appRoutes: string, moduleRoutes: string): string[] {
  const owned = new Set<string>()
  const walk = (relative: string) => {
    for (const entry of readdirSync(path.join(moduleRoutes, relative), { withFileTypes: true })) {
      const next = path.join(relative, entry.name)
      if (entry.isDirectory() && existsSync(path.join(appRoutes, next))) {
        walk(next)
        continue
      }
      if (!entry.isDirectory() && !/\.[jt]sx?$/.test(entry.name)) continue
      if (/\.(test|spec)\.[jt]sx?$/.test(entry.name)) continue
      const name = entry.isDirectory() ? entry.name : entry.name.replace(/\.[jt]sx?$/, '')
      if (!entry.isDirectory() && (name === 'index' || name === 'route')) continue
      const segments = [...relative.split(path.sep), name].filter((s) => s && !s.startsWith('_'))
      if (segments.length > 0) owned.add(`/${segments.join('/')}`)
    }
  }
  walk('')
  return [...owned].sort()
}

function apiIdentifiers(moduleRoot: string): string[] {
  return ['src/api/client.ts', 'src/api/queries.ts']
    .map((file) => path.join(moduleRoot, file))
    .filter((file) => existsSync(file))
    .flatMap((file) =>
      [...readFileSync(file, 'utf8').matchAll(/^export (?:const|function) (\w+)/gm)].map(
        ([, name]) => name,
      ),
    )
    .sort()
}

const escaped = (term: string) => term.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')

/** Only the package import is spelled out; everything else follows from the module packages. */
function forbidden(
  builds: readonly ModuleBuild[],
  manifests: readonly ModuleManifest[],
  appRoutes: string,
): [string, RegExp][] {
  const rules: [string, RegExp][] = [['imports a module package', /@green-ecolution\/module-/]]
  // An empty alternation would match every file.
  const derive = (label: string, terms: readonly string[], pattern: (any: string) => RegExp) => {
    if (terms.length > 0) rules.push([label, pattern(terms.map(escaped).join('|'))])
  }
  const keys = builds.map(({ key }) => key)
  derive('names a module package path', keys, (any) => new RegExp(`modules/(${any})\\b`))
  derive(
    'links a module route',
    builds.flatMap(({ root, routesDirectory }) =>
      routesDirectory ? ownedRoutePrefixes(appRoutes, path.join(root, routesDirectory)) : [],
    ),
    (any) => new RegExp(`['"\`](${any})\\b`),
  )
  derive(
    'uses a module api',
    builds.flatMap(({ root }) => apiIdentifiers(root)),
    (any) => new RegExp(`\\b(${any})\\b`),
  )
  derive('names a module', keys, (any) => new RegExp(`['"](${any})['"]`))
  derive(
    'names a module resource',
    manifests.flatMap(({ resources }) => (resources ?? []).map(({ key }) => key)),
    (any) => new RegExp(`\\b(${any}):(${ACTIONS})\\b`),
  )
  return rules
}

function sourceFiles(dir: string): string[] {
  return readdirSync(dir).flatMap((entry) => {
    const full = path.join(dir, entry)
    if (statSync(full).isDirectory()) return full === TEST_SUPPORT ? [] : sourceFiles(full)
    return /\.(ts|tsx)$/.test(entry) && !/\.(test|spec)\.tsx?$/.test(entry) ? [full] : []
  })
}

// Every relative path literal counts, not only import specifiers: a glob or a
// `new URL(…)` reaches a module package just as well.
const RELATIVE_PATH = /['"`](\.{1,2}\/[^'"`]*)['"`]/g

function reachesModulePackage(file: string, source: string, modulePackages: string): boolean {
  return [...source.matchAll(RELATIVE_PATH)].some(([, specifier]) => {
    const fixedPart = specifier.replace(/[*?{[].*$/, '')
    const target = path.resolve(path.dirname(file), fixedPart)
    return target === modulePackages || target.startsWith(modulePackages + path.sep)
  })
}

describe('core boundary', () => {
  const files = sourceFiles(SRC).filter((f) => !COMPOSITION_ROOTS.has(path.relative(SRC, f)))

  it.each(forbidden(MODULE_BUILDS, moduleManifests, path.join(SRC, 'routes')))(
    'no core file %s',
    (_label, pattern) => {
      const offenders = files.filter((f) => pattern.test(readFileSync(f, 'utf8')))
      expect(offenders.map((f) => path.relative(SRC, f))).toEqual([])
    },
  )

  it('no core file reaches a module package by relative path', () => {
    const offenders = files.filter((f) =>
      reachesModulePackage(f, readFileSync(f, 'utf8'), MODULE_PACKAGES),
    )
    expect(offenders.map((f) => path.relative(SRC, f))).toEqual([])
  })
})

describe('derived terms', () => {
  let dir: string

  beforeEach(() => {
    dir = mkdtempSync(path.join(os.tmpdir(), 'boundary-'))
  })
  afterEach(() => {
    rmSync(dir, { recursive: true, force: true })
  })

  const file = (relative: string, content = '') => {
    const full = path.join(dir, relative)
    mkdirSync(path.dirname(full), { recursive: true })
    writeFileSync(full, content)
  }

  it('names the first segment a module adds below a shared path', () => {
    file('app/_protected/map/index.tsx')
    file('module/_protected/map/thing/index.tsx')
    file('module/_protected/things/route.tsx')
    file('module/_protected/things/$thingId/index.tsx')
    file('module/_protected/things/_formular/new.tsx')

    expect(ownedRoutePrefixes(path.join(dir, 'app'), path.join(dir, 'module'))).toEqual([
      '/map/thing',
      '/things',
    ])
  })

  it('reads a route file directly below a shared path', () => {
    file('app/_protected/map/index.tsx')
    file('module/_protected/map/thing.tsx')
    file('module/_protected/map/thing.test.tsx')

    expect(ownedRoutePrefixes(path.join(dir, 'app'), path.join(dir, 'module'))).toEqual([
      '/map/thing',
    ])
  })

  it('collects the exported names of the api files and tolerates a module without them', () => {
    file('with/src/api/client.ts', 'export const thingApi = 1\nconst hidden = 2\n')
    file('with/src/api/queries.ts', 'export function thingQueries() {}\n')
    file('without/src/manifest.tsx')

    expect(apiIdentifiers(path.join(dir, 'with'))).toEqual(['thingApi', 'thingQueries'])
    expect(apiIdentifiers(path.join(dir, 'without'))).toEqual([])
  })

  it('sees every relative path that lands in a module package, not only imports', () => {
    const core = path.join(dir, 'app/src/modules/loader.ts')
    const packages = path.join(dir, 'modules')

    expect(
      reachesModulePackage(core, "import.meta.glob('../../../modules/*/src/x.ts')", packages),
    ).toBe(true)
    expect(reachesModulePackage(core, "import x from '../../../modules/a/src/x'", packages)).toBe(
      true,
    )
    expect(
      reachesModulePackage(core, "new URL('../../../modules/a/x.png', import.meta.url)", packages),
    ).toBe(true)
    expect(reachesModulePackage(core, "import y from '../lib/y'", packages)).toBe(false)
    expect(reachesModulePackage(core, "import.meta.glob('../locales/*/index.ts')", packages)).toBe(
      false,
    )
  })

  it('forbids nothing derived when there is no module', () => {
    expect(forbidden([], [], path.join(dir, 'app')).map(([label]) => label)).toEqual([
      'imports a module package',
    ])
  })

  it('derives the rules of a module from its package and its manifest', () => {
    file('app/_protected/map/index.tsx')
    file('things/src/routes/_protected/things/index.tsx')
    file('things/src/api/client.ts', 'export const thingApi = 1\n')
    const manifest = { key: 'things', resources: [{ key: 'thing' }] } as unknown as ModuleManifest
    const rules = new Map(
      forbidden(
        [{ key: 'things', root: path.join(dir, 'things'), routesDirectory: 'src/routes' }],
        [manifest],
        path.join(dir, 'app'),
      ),
    )

    expect(rules.get('names a module package path')?.test("from '../../modules/things/src'")).toBe(
      true,
    )
    expect(rules.get('links a module route')?.test("to: '/things'")).toBe(true)
    expect(rules.get('links a module route')?.test("to: '/sensors'")).toBe(false)
    expect(rules.get('uses a module api')?.test('thingApi.list()')).toBe(true)
    expect(rules.get('names a module')?.test("key === 'things'")).toBe(true)
    expect(rules.get('names a module')?.test("key === 'thingsx'")).toBe(false)
    expect(rules.get('names a module resource')?.test("['thing:read']")).toBe(true)
  })
})
