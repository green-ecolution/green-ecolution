import { expect, it } from 'vitest'
import de from '@/locales/de'
import en from '@/locales/en'
import { uiDe, uiEn } from '@green-ecolution/ui'
import { NAMESPACES } from './languages'
import { ALL_NAMESPACES } from './index'
import { moduleManifests } from '@/modules/registry'

function flatten(value: unknown, prefix = ''): string[] {
  if (typeof value === 'string') return [prefix]
  if (value === null || typeof value !== 'object') return []
  return Object.entries(value).flatMap(([key, child]) =>
    flatten(child, prefix ? `${prefix}.${key}` : key),
  )
}

it('registers every catalog namespace', () => {
  const registered = new Set<string>(NAMESPACES)
  for (const namespace of Object.keys(de)) {
    expect(registered.has(namespace)).toBe(true)
  }
  expect(registered.has('ui')).toBe(true)
})

it('registers every module namespace without colliding with the core', () => {
  const registered = new Set<string>(ALL_NAMESPACES)
  const core = new Set<string>(NAMESPACES)
  const moduleNamespaces = moduleManifests.flatMap((manifest) => manifest.i18n.namespaces)
  expect(moduleNamespaces).toContain('irrigation')
  expect(ALL_NAMESPACES).toHaveLength(NAMESPACES.length + moduleNamespaces.length)
  for (const manifest of moduleManifests) {
    for (const namespace of manifest.i18n.namespaces) {
      expect(registered.has(namespace), namespace).toBe(true)
      expect(core.has(namespace), `${namespace} collides with a core namespace`).toBe(false)
    }
  }
})

it('ships the same module namespaces and keys in German and English', async () => {
  for (const manifest of moduleManifests) {
    const [deModule, enModule] = await Promise.all([
      manifest.i18n.load('de'),
      manifest.i18n.load('en'),
    ])
    expect(Object.keys(deModule).sort()).toEqual([...manifest.i18n.namespaces].sort())
    expect(Object.keys(enModule).sort()).toEqual(Object.keys(deModule).sort())
    for (const [namespace, catalog] of Object.entries(deModule)) {
      expect(flatten(enModule[namespace]).sort(), namespace).toEqual(flatten(catalog).sort())
    }
  }
})

it('has the same key set in German and English, per namespace', () => {
  for (const [namespace, catalog] of Object.entries(de)) {
    const other = (en as Record<string, unknown>)[namespace]
    expect(flatten(other).sort(), `namespace ${namespace}`).toEqual(flatten(catalog).sort())
  }
  expect(flatten(uiEn).sort()).toEqual(flatten(uiDe).sort())
})

it('leaves no empty translation', () => {
  const empty = [...Object.entries(de), ...Object.entries(en)].flatMap(([namespace, catalog]) =>
    flatten(catalog)
      .filter((path) => {
        const value = path
          .split('.')
          .reduce<unknown>((node, part) => (node as Record<string, unknown>)?.[part], catalog)
        return typeof value === 'string' && value.trim() === ''
      })
      .map((path) => `${namespace}:${path}`),
  )
  expect(empty).toEqual([])
})
