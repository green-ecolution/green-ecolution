import type { UiCatalog } from '@green-ecolution/ui'
import type de from '@/locales/de'
import type irrigationDe from '@green-ecolution/module-irrigation/locales/de'
import type wasteDe from '@green-ecolution/module-waste/locales/de'

// The German catalog is the source of truth for the key space: it is the
// fallback language and therefore always complete. `ui` is added separately
// because it ships from the component package, not from a locale barrel.
declare module 'i18next' {
  interface CustomTypeOptions {
    defaultNS: 'common'
    resources: typeof de & { ui: UiCatalog } & typeof irrigationDe & typeof wasteDe
  }
}
