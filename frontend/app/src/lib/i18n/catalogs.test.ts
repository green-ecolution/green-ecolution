import { expect, it } from 'vitest'
import de from '@/locales/de'
import en from '@/locales/en'
import { uiDe, uiEn } from '@green-ecolution/ui'
import { NAMESPACES } from './languages'

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
