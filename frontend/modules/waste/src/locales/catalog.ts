import type de from './de'

declare module '@/lib/i18n/catalogs' {
  interface Catalogs {
    waste: typeof de
  }
}
