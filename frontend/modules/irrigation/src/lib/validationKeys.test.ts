import { expect, it } from 'vitest'
import de from '../locales/de/irrigation.json'
import en from '../locales/en/irrigation.json'
import { IRRIGATION_VALIDATION_KEYS } from './validationKeys'

type Catalog = Record<string, unknown>

function textOf(catalog: Catalog, key: string): unknown {
  return ['validation', ...key.split('.')].reduce<unknown>(
    (node, part) => (node as Catalog | undefined)?.[part],
    catalog,
  )
}

const placeholders = (text: unknown) =>
  typeof text === 'string' ? [...text.matchAll(/\{[a-zA-Z0-9_]+\}/g)].map(([m]) => m).sort() : []

it.each(IRRIGATION_VALIDATION_KEYS)('%s has a German and an English text', (key) => {
  expect(textOf(de, key)).toEqual(expect.any(String))
  expect(textOf(en, key)).toEqual(expect.any(String))
})

it.each(IRRIGATION_VALIDATION_KEYS)('%s uses the same placeholders in both languages', (key) => {
  expect(placeholders(textOf(en, key))).toEqual(placeholders(textOf(de, key)))
})
