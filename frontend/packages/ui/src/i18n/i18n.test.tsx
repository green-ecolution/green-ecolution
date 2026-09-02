import { render, screen } from '@testing-library/react'
import { describe, expect, it } from 'vitest'
import { fallbackTranslate, uiDe, uiEn, UiTextProvider, useUiText } from './index'

function Probe() {
  const { t, locale } = useUiText()
  return (
    <>
      <span data-testid="close">{t('dialog.close')}</span>
      <span data-testid="count">{t('combobox.selectedCount', { count: 3 })}</span>
      <span data-testid="locale">{locale}</span>
    </>
  )
}

describe('fallbackTranslate', () => {
  it('resolves a dotted path', () => {
    expect(fallbackTranslate(uiDe)('dialog.close')).toBe('Schließen')
  })

  it('interpolates single-brace placeholders', () => {
    expect(fallbackTranslate(uiDe)('combobox.selectedCount', { count: 3 })).toBe('3 ausgewählt')
  })

  it('returns the key when it is unknown', () => {
    // @ts-expect-error unknown key must not type-check
    expect(fallbackTranslate(uiDe)('dialog.nope')).toBe('dialog.nope')
  })

  it('has the same key set in German and English', () => {
    const paths = (value: unknown, prefix = ''): string[] =>
      typeof value === 'string'
        ? [prefix]
        : Object.entries(value as object).flatMap(([k, v]) =>
            paths(v, prefix ? `${prefix}.${k}` : k),
          )
    expect(paths(uiEn).sort()).toEqual(paths(uiDe).sort())
  })
})

describe('UiTextProvider', () => {
  it('defaults to the German catalog with no provider around', () => {
    render(<Probe />)
    expect(screen.getByTestId('close')).toHaveTextContent('Schließen')
    expect(screen.getByTestId('count')).toHaveTextContent('3 ausgewählt')
    expect(screen.getByTestId('locale')).toHaveTextContent('de')
  })

  it('uses the injected translator when one is provided', () => {
    render(
      <UiTextProvider t={fallbackTranslate(uiEn)} locale="en">
        <Probe />
      </UiTextProvider>,
    )
    expect(screen.getByTestId('close')).toHaveTextContent('Close')
    expect(screen.getByTestId('locale')).toHaveTextContent('en')
  })
})
