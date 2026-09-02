import { UiTextProvider, type UiTranslate } from '@green-ecolution/ui'
import { useCallback, type ReactNode } from 'react'
import { useTranslation } from 'react-i18next'
import { intlLocale } from './format'

/**
 * Hands the app's `ui` namespace to the component package, which knows keys but
 * deliberately not i18next.
 */
export function UiTextBridge({ children }: { children: ReactNode }) {
  const { t, i18n } = useTranslation('ui')

  // Load-bearing: UiTextProvider memoises on this identity, so a fresh
  // function on every render would re-render every consuming component.
  const translate = useCallback<UiTranslate>((key, params) => t(key, params), [t])

  return (
    <UiTextProvider t={translate} locale={intlLocale(i18n.language)}>
      {children}
    </UiTextProvider>
  )
}
