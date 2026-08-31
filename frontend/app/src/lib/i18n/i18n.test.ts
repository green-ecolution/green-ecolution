import { beforeEach, describe, expect, it } from 'vitest'
import { createI18n, getI18n, switchLanguage } from './index'

describe('i18n instance', () => {
  beforeEach(() => {
    localStorage.clear()
  })

  it('resolves a German key by default', async () => {
    const i18n = await createI18n()
    expect(i18n.t('common:actions.save')).toBe('Speichern')
  })

  it('interpolates with single braces, not double', async () => {
    const i18n = await createI18n()
    expect(i18n.t('common:pagination.pageOf', { page: 2, total: 7 })).toBe('Seite 2 von 7')
  })

  it('serves English after switching and keeps German reachable', async () => {
    await createI18n()
    await switchLanguage('en')
    expect(getI18n().t('common:actions.save')).toBe('Save')
    await switchLanguage('de')
    expect(getI18n().t('common:actions.save')).toBe('Speichern')
  })

  it('rejects an unknown language and falls back to German', async () => {
    const i18n = await createI18n()
    await i18n.changeLanguage('fr')
    expect(i18n.t('common:actions.save')).toBe('Speichern')
  })
})
