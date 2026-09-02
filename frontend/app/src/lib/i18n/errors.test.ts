import { beforeEach, expect, it } from 'vitest'
import { createI18n } from './index'
import de from '@/locales/de/errors.json'
import en from '@/locales/en/errors.json'

beforeEach(() => {
  localStorage.clear()
})

function flatten(value: unknown, prefix = ''): string[] {
  if (typeof value === 'string') return [prefix]
  if (value === null || typeof value !== 'object') return []
  return Object.entries(value).flatMap(([key, child]) =>
    flatten(child, prefix ? `${prefix}.${key}` : key),
  )
}

/** Same traversal as `flatten`, but keeps the leaf string alongside its key. */
function flattenEntries(value: unknown, prefix = ''): [string, string][] {
  if (typeof value === 'string') return [[prefix, value]]
  if (value === null || typeof value !== 'object') return []
  return Object.entries(value).flatMap(([key, child]) =>
    flattenEntries(child, prefix ? `${prefix}.${key}` : key),
  )
}

/** Every catalog leaf, tagged with its language, for checks that scan both. */
function taggedEntries(): [string, string][] {
  return [
    ...flattenEntries(de).map(([key, value]): [string, string] => [`de:${key}`, value]),
    ...flattenEntries(en).map(([key, value]): [string, string] => [`en:${key}`, value]),
  ]
}

const PLACEHOLDER = /\{[a-zA-Z0-9_]+\}/g

function placeholdersIn(value: string): Set<string> {
  return new Set(value.match(PLACEHOLDER) ?? [])
}

it('has the same key set in German and English', () => {
  expect(flatten(en).sort()).toEqual(flatten(de).sort())
})

it('keeps the code and status key space intact', () => {
  const keys = flatten(de)
  expect(keys).toContain('code.conflict.tree_in_cluster')
  expect(keys).toContain('code.organization_mismatch.trees_vs_cluster')
  expect(keys).toContain('status.409')
  expect(keys).toContain('offline')
  expect(keys).toContain('unknown')
})

it('interpolates the unknown-status placeholder', async () => {
  const i18n = await createI18n()
  expect(i18n.t('errors:status.unknown', { status: 418 })).toBe(
    'Der Server hat mit Status 418 geantwortet.',
  )
})

// Same class of regression checks as validation.test.ts: transcribing 52
// entries by hand is exactly where a mismatched placeholder or a mangled
// umlaut slips through key-set parity alone.

it('uses the same placeholders in German and English for every key', () => {
  const deEntries = new Map(flattenEntries(de))
  const enEntries = new Map(flattenEntries(en))
  const mismatches: string[] = []
  for (const [key, deValue] of deEntries) {
    const enValue = enEntries.get(key)
    // A key missing from one catalog is reported by the key-set-parity test.
    if (enValue === undefined) continue
    const deSet = placeholdersIn(deValue)
    const enSet = placeholdersIn(enValue)
    const same = deSet.size === enSet.size && [...deSet].every((p) => enSet.has(p))
    if (!same) {
      mismatches.push(`${key}: de=${[...deSet].sort().join(',')} en=${[...enSet].sort().join(',')}`)
    }
  }
  expect(mismatches).toEqual([])
})

it('has no empty or whitespace-only values', () => {
  const blank = taggedEntries()
    .filter(([, value]) => value.trim().length === 0)
    .map(([key]) => key)
  expect(blank).toEqual([])
})

it('has no leftover template-literal syntax from the old renderer form', () => {
  const leftover = taggedEntries()
    .filter(([, value]) => value.includes('${'))
    .map(([key]) => key)
  expect(leftover).toEqual([])
})

it('uses single-brace interpolation only', () => {
  const doubled = taggedEntries()
    .filter(([, value]) => value.includes('{{') || value.includes('}}'))
    .map(([key]) => key)
  expect(doubled).toEqual([])
})
