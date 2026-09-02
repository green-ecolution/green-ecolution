import type { Locale } from 'date-fns'
import { de, enGB } from 'date-fns/locale'
import { languageOf, type Language } from './languages'

// en-GB, not en-US: the app is used in Germany, so day-month-year ordering and
// 24-hour time stay correct when the interface language changes.
const DATE_FNS: Record<Language, Locale> = { de, en: enGB }
const INTL: Record<Language, string> = { de: 'de-DE', en: 'en-GB' }

export const dateFnsLocale = (tag: string): Locale => DATE_FNS[languageOf(tag)]

export const intlLocale = (tag: string): string => INTL[languageOf(tag)]
