import { render, screen } from '@testing-library/react'
import { Dialog, DialogContent, DialogTrigger } from '@green-ecolution/ui'
import { I18nextProvider } from 'react-i18next'
import { beforeEach, expect, it } from 'vitest'
import { createI18n } from './index'
import { UiTextBridge } from './UiTextBridge'

beforeEach(() => {
  localStorage.clear()
})

it('serves the ui namespace from the package catalog', async () => {
  const i18n = await createI18n()
  expect(i18n.t('ui:dialog.close')).toBe('Schließen')
  await i18n.changeLanguage('en')
  expect(i18n.t('ui:dialog.close')).toBe('Close')
})

it('passes the app translator down to package components', async () => {
  const i18n = await createI18n()
  await i18n.changeLanguage('en')
  render(
    <I18nextProvider i18n={i18n}>
      <UiTextBridge>
        <Dialog open>
          <DialogTrigger>open</DialogTrigger>
          <DialogContent>content</DialogContent>
        </Dialog>
      </UiTextBridge>
    </I18nextProvider>,
  )
  expect(await screen.findByText('Close')).toBeInTheDocument()
})
