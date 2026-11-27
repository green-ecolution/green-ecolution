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
import { VALIDATION_KEYS } from '@green-ecolution/domain-wasm'
import { afterEach, beforeEach, describe, expect, it } from 'vitest'
import { MODULE_BUILDS } from '../../modules.config'
import type { ModuleBuild } from './buildConfig'
import { moduleManifests } from './registry'
import type { ModuleManifest } from './types'

const SRC = path.resolve(import.meta.dirname, '..')
const LOCALES = path.join(SRC, 'locales')
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
  // As a literal, a key segment or a property: `'water'`, `kind.water`, `capacities.water`.
  derive(
    'names a capacity kind of a module',
    capacityKinds(manifests),
    (any) => new RegExp(`['"\`.](${any})\\b`),
  )
  return rules
}

const capacityKinds = (manifests: readonly ModuleManifest[]) =>
  manifests.flatMap(({ capacities }) => (capacities ?? []).map(({ kind }) => kind))

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

interface Catalog {
  [key: string]: string | Catalog
}

function leafKeys(node: Catalog | string | undefined, prefix = ''): string[] {
  if (node === undefined) return []
  if (typeof node === 'string') return [prefix]
  return Object.entries(node).flatMap(([key, child]) =>
    leafKeys(child, prefix ? `${prefix}.${key}` : key),
  )
}

function textAt(catalog: Catalog, key: string): Catalog | string | undefined {
  return key
    .split('.')
    .reduce<Catalog | string | undefined>(
      (node, part) => (typeof node === 'object' ? node[part] : undefined),
      catalog,
    )
}

/**
 * The sections of the validation texts that belong to the modules: for every
 * module key the shortest prefix no core key starts with. A core key a module
 * words for itself yields none, so that stays allowed.
 */
function moduleOwnedSections(moduleKeys: readonly string[], coreKeys: readonly string[]): string[] {
  const inCore = (prefix: string) =>
    coreKeys.some((key) => key === prefix || key.startsWith(`${prefix}.`))
  const owned = moduleKeys.flatMap((key) => {
    const parts = key.split('.')
    const prefixes = parts.map((_, index) => parts.slice(0, index + 1).join('.'))
    return prefixes.filter((prefix) => !inCore(prefix)).slice(0, 1)
  })
  return [...new Set(owned)].sort()
}

function moduleCatalogs(moduleRoot: string, language: string): Catalog[] {
  const dir = path.join(moduleRoot, 'src/locales', language)
  if (!existsSync(dir)) return []
  return readdirSync(dir)
    .filter((entry) => entry.endsWith('.json'))
    .map((entry) => JSON.parse(readFileSync(path.join(dir, entry), 'utf8')) as Catalog)
}

const coreCatalog = (language: string, namespace: string) =>
  JSON.parse(readFileSync(path.join(LOCALES, language, `${namespace}.json`), 'utf8')) as Catalog

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

  const languages = readdirSync(LOCALES)

  it.each(languages)('the %s validation catalog holds no section of a module', (language) => {
    const moduleKeys = MODULE_BUILDS.flatMap(({ root }) =>
      moduleCatalogs(root, language).flatMap(({ validation }) => leafKeys(validation)),
    )
    const core = coreCatalog(language, 'validation')
    const offenders = moduleOwnedSections(moduleKeys, VALIDATION_KEYS).filter(
      (section) => textAt(core, section) !== undefined,
    )
    expect(offenders.map((section) => `validation.json: ${section}`)).toEqual([])
  })

  it.each(languages)('no %s core catalog names a capacity kind of a module', (language) => {
    const kinds = new Set(capacityKinds(moduleManifests))
    const offenders = readdirSync(path.join(LOCALES, language))
      .filter((file) => file.endsWith('.json'))
      .flatMap((file) =>
        leafKeys(coreCatalog(language, file.replace(/\.json$/, '')))
          .filter((key) => key.split('.').some((segment) => kinds.has(segment)))
          .map((key) => `${file}: ${key}`),
      )
    expect(offenders).toEqual([])
  })

  it.each(languages)('the %s settings catalog names no role template of a module', (language) => {
    const core = coreCatalog(language, 'settings')
    const offenders = moduleManifests
      .flatMap(({ roleTemplates }) => (roleTemplates ?? []).map(({ key }) => key))
      .filter((key) => textAt(core, `roles.templateName.${key}`) !== undefined)
    expect(offenders.map((key) => `settings.json: roles.templateName.${key}`)).toEqual([])
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

  it("takes a section for a module's own from the first part the core does not know", () => {
    const core = ['plan.stop_ids.empty', 'settings.map_view.out_of_range']

    expect(
      moduleOwnedSections(
        ['thing.name.empty', 'thing.name.tooLong', 'settings.thing_limit.out_of_range'],
        core,
      ),
    ).toEqual(['settings.thing_limit', 'thing'])
  })

  it('leaves a core key a module words for itself to the core', () => {
    expect(moduleOwnedSections(['plan.stop_ids.empty'], ['plan.stop_ids.empty'])).toEqual([])
  })

  it('reads the catalogs of a module per language and tolerates a module without them', () => {
    file('with/src/locales/de/things.json', '{"validation":{"thing":{"empty":"x"}}}')
    file('with/src/locales/de/index.ts')
    file('without/src/manifest.tsx')

    expect(
      moduleCatalogs(path.join(dir, 'with'), 'de').flatMap(({ validation }) =>
        leafKeys(validation),
      ),
    ).toEqual(['thing.empty'])
    expect(moduleCatalogs(path.join(dir, 'with'), 'en')).toEqual([])
    expect(moduleCatalogs(path.join(dir, 'without'), 'de')).toEqual([])
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
