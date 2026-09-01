export const SUPPORTED_LANGUAGES = ['de', 'en'] as const

export type Language = (typeof SUPPORTED_LANGUAGES)[number]

export const FALLBACK_LANGUAGE: Language = 'de'

/**
 * Grows with every extraction task. i18next only preloads what is listed here,
 * so a namespace missing from this array resolves to its raw key.
 */
export const NAMESPACES = ['common', 'validation', 'errors', 'ui'] as const

export const DEFAULT_NAMESPACE = 'common'

export function isLanguage(value: string): value is Language {
  return (SUPPORTED_LANGUAGES as readonly string[]).includes(value)
}

/** `de-DE` and `en-GB` both reduce to their base language. */
export function languageOf(tag: string): Language {
  const base = tag.split('-')[0]
  return isLanguage(base) ? base : FALLBACK_LANGUAGE
}
