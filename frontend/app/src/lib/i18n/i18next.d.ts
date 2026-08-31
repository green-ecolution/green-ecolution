import type de from '@/locales/de'

// The German catalog is the source of truth for the key space: it is the
// fallback language and therefore always complete.
declare module 'i18next' {
  interface CustomTypeOptions {
    defaultNS: 'common'
    resources: typeof de
  }
}
