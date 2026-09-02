import * as React from 'react'
import { uiDe, type UiTranslate } from './catalog'
import { fallbackTranslate } from './fallback'

interface UiTextContextValue {
  t: UiTranslate
  /** BCP 47 tag, used for `Intl`-derived copy such as month names. */
  locale: string
}

const defaultValue: UiTextContextValue = { t: fallbackTranslate(uiDe), locale: 'de' }

const UiTextContext = React.createContext<UiTextContextValue>(defaultValue)

export interface UiTextProviderProps {
  t?: UiTranslate
  locale?: string
  children: React.ReactNode
}

export function UiTextProvider({ t, locale, children }: UiTextProviderProps) {
  const value = React.useMemo<UiTextContextValue>(
    () => ({ t: t ?? defaultValue.t, locale: locale ?? defaultValue.locale }),
    [t, locale],
  )
  return <UiTextContext.Provider value={value}>{children}</UiTextContext.Provider>
}

export function useUiText(): UiTextContextValue {
  return React.useContext(UiTextContext)
}
