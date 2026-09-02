import i18next, { type i18n as I18n } from 'i18next'
import LanguageDetector from 'i18next-browser-languagedetector'
import { initReactI18next } from 'react-i18next'
import { ensureLanguage } from './load'
import {
  DEFAULT_NAMESPACE,
  FALLBACK_LANGUAGE,
  NAMESPACES,
  SUPPORTED_LANGUAGES,
  languageOf,
  type Language,
} from './languages'

let current: I18n | undefined

export async function createI18n(): Promise<I18n> {
  const instance = i18next.createInstance()

  await instance
    .use(LanguageDetector)
    .use(initReactI18next)
    .init({
      fallbackLng: FALLBACK_LANGUAGE,
      supportedLngs: SUPPORTED_LANGUAGES,
      nonExplicitSupportedLngs: true,
      ns: NAMESPACES,
      defaultNS: DEFAULT_NAMESPACE,
      resources: {},
      detection: {
        order: ['querystring', 'localStorage', 'navigator'],
        lookupQuerystring: 'lng',
        lookupLocalStorage: 'gec.language',
        caches: ['localStorage'],
      },
      // Single braces, because both pre-existing catalogs already use `{min}`,
      // `{max}` and `{status}` and are wire-contract with the backend.
      interpolation: { prefix: '{', suffix: '}', escapeValue: false },
      returnNull: false,
      saveMissing: import.meta.env.DEV,
      missingKeyHandler: (_lngs, ns, key, fallbackValue) => {
        const message = `i18n: missing key ${ns}:${key}`
        // A key present in German but not yet in English renders via the
        // fallback and is expected mid-extraction. Missing everywhere is a bug.
        if (fallbackValue && fallbackValue !== key) console.warn(message)
        else throw new Error(message)
      },
    })

  await ensureLanguage(instance, languageOf(instance.resolvedLanguage ?? instance.language))
  current = instance
  return instance
}

export function getI18n(): I18n {
  if (!current) throw new Error('i18n: createI18n() has not completed yet')
  return current
}

/** Loads the target language's chunk before switching, so no render sees raw keys. */
export async function switchLanguage(lng: Language): Promise<void> {
  const instance = getI18n()
  await ensureLanguage(instance, lng)
  await instance.changeLanguage(lng)
}
