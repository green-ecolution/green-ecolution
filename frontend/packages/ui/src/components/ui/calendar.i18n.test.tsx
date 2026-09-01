import { render, screen } from '@testing-library/react'
import { expect, it } from 'vitest'
import { UiTextProvider, fallbackTranslate, uiEn } from '../../i18n'
import { Calendar } from './calendar'

it('labels months in German by default', () => {
  // Calendar's own display month is seeded from `selected`, not `month` —
  // there is no controlled `month` prop on this hand-rolled wrapper.
  render(<Calendar mode="single" selected={new Date(2026, 2, 1)} />)
  expect(screen.getByRole('combobox', { name: 'Monat auswählen' })).toHaveDisplayValue('März')
})

it('labels months in English under an English provider', () => {
  render(
    <UiTextProvider t={fallbackTranslate(uiEn)} locale="en">
      <Calendar mode="single" selected={new Date(2026, 2, 1)} />
    </UiTextProvider>,
  )
  expect(screen.getByRole('combobox', { name: 'Select month' })).toHaveDisplayValue('March')
})
