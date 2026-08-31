import { expect, it } from 'vitest'
import { createI18n } from './index'

it('accepts a known key and rejects an unknown one', async () => {
  const i18n = await createI18n()
  expect(i18n.t('common:actions.save')).toBe('Speichern')

  // Compile-time probe, never executed. `tsc -b --noEmit` fails if the
  // ts-expect-error directive below becomes unused, which happens exactly
  // when the key space stops being typed.
  const probe = () => {
    // @ts-expect-error unknown translation key must not type-check
    i18n.t('common:actions.doesNotExist')
  }
  expect(typeof probe).toBe('function')
})
