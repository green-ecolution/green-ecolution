import type { CatalogResources } from './catalogs'

// The German catalogs are the source of truth for the key space: German is the
// fallback language and therefore always complete.
declare module 'i18next' {
  interface CustomTypeOptions {
    defaultNS: 'common'
    resources: CatalogResources
  }
}
