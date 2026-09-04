import { render, screen } from '@testing-library/react'
import { I18nextProvider } from 'react-i18next'
import { afterEach, describe, expect, it } from 'vitest'
import { getI18n, switchLanguage } from '@/lib/i18n'
import { handbookIndex } from '@/lib/handbook'
import LanguageFallbackNotice from './LanguageFallbackNotice'

const renderNotice = () =>
  render(
    <I18nextProvider i18n={getI18n()}>
      <LanguageFallbackNotice />
    </I18nextProvider>,
  )

// switchLanguage mutates the shared instance from src/test/setup.ts.
afterEach(async () => {
  await switchLanguage('de')
})

describe('LanguageFallbackNotice', () => {
  it('stays silent while the interface language matches the chapters', async () => {
    expect(handbookIndex.language).toBe('de')
    await switchLanguage('de')

    const { container } = renderNotice()

    expect(container).toBeEmptyDOMElement()
  })

  it('explains the German chapters to a reader on an English interface', async () => {
    await switchLanguage('en')

    renderNotice()

    expect(screen.getByText(/not available in English yet/i)).toBeInTheDocument()
  })
})
