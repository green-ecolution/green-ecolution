import { render, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { I18nextProvider } from 'react-i18next'
import { afterEach, beforeEach, expect, it, vi } from 'vitest'
import { getI18n, switchLanguage } from '@/lib/i18n'
import LanguageSwitcher from './LanguageSwitcher'

const showToast = vi.fn()
vi.mock('@/hooks/createToast', () => ({ default: () => showToast }))

// Wraps the real switchLanguage so one test can force it to reject (simulating
// a stale chunk 404 after a deploy) while the others keep real behaviour.
vi.mock('@/lib/i18n', async (importOriginal) => {
  const actual = await importOriginal<typeof import('@/lib/i18n')>()
  return { ...actual, switchLanguage: vi.fn(actual.switchLanguage) }
})

beforeEach(() => {
  localStorage.clear()
  showToast.mockClear()
  vi.mocked(switchLanguage).mockClear()
})

// switchLanguage mutates the shared instance from src/test/setup.ts; reset it
// so a later test doesn't inherit English from this one.
afterEach(async () => {
  await getI18n().changeLanguage('de')
})

it('switches the interface language without a reload', async () => {
  const i18n = getI18n()
  render(
    <I18nextProvider i18n={i18n}>
      <LanguageSwitcher />
    </I18nextProvider>,
  )

  expect(screen.getByRole('radiogroup', { name: 'Sprache' })).toBeInTheDocument()

  await userEvent.click(screen.getByRole('radio', { name: 'Englisch' }))

  // The component only re-renders once react-i18next's languageChanged event
  // fires, which is also the point switchLanguage's promise has resolved —
  // so wait for the re-render before asserting on the instance itself.
  expect(await screen.findByRole('radiogroup', { name: 'Language' })).toBeInTheDocument()
  expect(i18n.language).toBe('en')
})

it('persists the choice to localStorage', async () => {
  const i18n = getI18n()
  render(
    <I18nextProvider i18n={i18n}>
      <LanguageSwitcher />
    </I18nextProvider>,
  )

  await userEvent.click(screen.getByRole('radio', { name: 'Englisch' }))

  expect(localStorage.getItem('gec.language')).toBe('en')
})

it('toasts and keeps the previous language when the language chunk fails to load', async () => {
  vi.mocked(switchLanguage).mockRejectedValueOnce(new Error('Failed to fetch dynamically imported module'))
  // eslint-disable-next-line @typescript-eslint/no-empty-function
  const consoleError = vi.spyOn(console, 'error').mockImplementation(() => {})
  const i18n = getI18n()
  const languageBeforeClick = i18n.language

  render(
    <I18nextProvider i18n={i18n}>
      <LanguageSwitcher />
    </I18nextProvider>,
  )

  await userEvent.click(screen.getByRole('radio', { name: 'Englisch' }))

  await waitFor(() =>
    expect(showToast).toHaveBeenCalledWith(
      'Die Sprache konnte nicht umgestellt werden. Bitte versuche es erneut.',
      'error',
    ),
  )
  expect(i18n.language).toBe(languageBeforeClick)
  expect(consoleError).toHaveBeenCalled()

  consoleError.mockRestore()
})
