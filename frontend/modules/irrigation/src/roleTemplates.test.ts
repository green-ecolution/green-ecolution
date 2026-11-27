import { expect, it } from 'vitest'
import { roleTemplateOf } from '@/modules/roleTemplates'
import de from './locales/de/irrigation.json'
import en from './locales/en/irrigation.json'
import manifest from './manifest'

it.each(['tree_care', 'route_planning'] as const)('names the delivered role %s', (key) => {
  expect(roleTemplateOf(key, [manifest])?.nameKey).toBe(`irrigation:roles.${key}`)
  expect(de.roles[key]).toEqual(expect.any(String))
  expect(en.roles[key]).toEqual(expect.any(String))
})
