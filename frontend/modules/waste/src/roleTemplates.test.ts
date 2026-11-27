import { Trash2 } from 'lucide-react'
import { expect, it } from 'vitest'
import { roleTemplateOf } from '@/modules/roleTemplates'
import de from './locales/de/waste.json'
import en from './locales/en/waste.json'
import manifest from './manifest'

it('names the delivered collection role and gives it the bin icon', () => {
  const template = roleTemplateOf('waste_collection', [manifest])

  expect(template?.nameKey).toBe('waste:roles.waste_collection')
  expect(template?.icon).toBe(Trash2)
  expect(de.roles.waste_collection).toBe('Abfallsammlung')
  expect(en.roles.waste_collection).toBe('Waste collection')
})
