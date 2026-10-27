import { readdirSync, readFileSync, statSync } from 'node:fs'
import path from 'node:path'
import { describe, expect, it } from 'vitest'

const SRC = path.resolve(import.meta.dirname, '..')
const MODULE_PACKAGES = path.resolve(SRC, '../../modules')
// Fixtures and MSW handlers are test code like the *.test files: they stand in
// for a backend that has irrigation registered.
const TEST_SUPPORT = path.join(SRC, 'test')
const COMPOSITION_ROOTS = new Set([
  'modules/registry.ts',
  'lib/i18n/i18next.d.ts',
  'routeTree.gen.ts',
])
const FORBIDDEN: [string, RegExp][] = [
  ['imports a module package', /@green-ecolution\/module-/],
  ['names a module package path', /modules\/irrigation/],
  [
    'links a module route',
    /['"`]\/(trees|treecluster|evaluations)\b|['"`]\/map\/(tree|treecluster|watering-plan)\b/,
  ],
  [
    'uses irrigation api',
    /\b(treeApi|clusterApi|evaluationApi|irrigationApi|treeQueries|clusterQueries)\b/,
  ],
  ['names the irrigation module', /['"]irrigation['"]/],
  [
    'names an irrigation resource',
    /\b(tree|tree_cluster|watering_plan):(read|create|update|delete)\b/,
  ],
]

function sourceFiles(dir: string): string[] {
  return readdirSync(dir).flatMap((entry) => {
    const full = path.join(dir, entry)
    if (statSync(full).isDirectory()) return full === TEST_SUPPORT ? [] : sourceFiles(full)
    return /\.(ts|tsx)$/.test(entry) && !/\.(test|spec)\.tsx?$/.test(entry) ? [full] : []
  })
}

const RELATIVE_IMPORT = /(?:from|import)\s*\(?\s*['"](\.{1,2}\/[^'"]*)['"]/g

function reachesModulePackage(file: string): boolean {
  return [...readFileSync(file, 'utf8').matchAll(RELATIVE_IMPORT)].some(([, specifier]) =>
    path.resolve(path.dirname(file), specifier).startsWith(MODULE_PACKAGES + path.sep),
  )
}

describe('core boundary', () => {
  const files = sourceFiles(SRC).filter((f) => !COMPOSITION_ROOTS.has(path.relative(SRC, f)))

  it.each(FORBIDDEN)('no core file %s', (_label, pattern) => {
    const offenders = files.filter((f) => pattern.test(readFileSync(f, 'utf8')))
    expect(offenders.map((f) => path.relative(SRC, f))).toEqual([])
  })

  it('no core file imports a module package by relative path', () => {
    const offenders = files.filter(reachesModulePackage)
    expect(offenders.map((f) => path.relative(SRC, f))).toEqual([])
  })
})
