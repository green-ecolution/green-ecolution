import type { Locale } from 'date-fns'
import { useTranslation } from 'react-i18next'
import { dateFnsLocale, intlLocale } from './format'

export function useDateLocale(): Locale {
  const { i18n } = useTranslation()
  return dateFnsLocale(i18n.language)
}

export function useNumberFormatter(options?: Intl.NumberFormatOptions): Intl.NumberFormat {
  const { i18n } = useTranslation()
  return new Intl.NumberFormat(intlLocale(i18n.language), options)
}
