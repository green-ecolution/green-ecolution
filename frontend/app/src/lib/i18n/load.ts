import { uiDe, uiEn } from '@green-ecolution/ui'
import type { i18n as I18n } from 'i18next'
import type { Language } from './languages'

// One `import()` per language literal, so Vite emits exactly one chunk per
// language instead of inlining both into the entry bundle.
async function bundlesFor(lng: Language): Promise<Record<string, object>> {
  return lng === 'en'
    ? (await import('../../locales/en')).default
    : (await import('../../locales/de')).default
}

const loaded = new WeakMap<I18n, Set<Language>>()

// `ui` is a static import from the package, already in memory for both
// languages at once (unlike the per-language chunks above), so it is
// registered for `de` and `en` together the first time an instance loads
// anything — not gated behind the `lng` being switched to.
function registerUiCatalog(instance: I18n): void {
  instance.addResourceBundle('de', 'ui', uiDe, true, true)
  instance.addResourceBundle('en', 'ui', uiEn, true, true)
}

export async function ensureLanguage(instance: I18n, lng: Language): Promise<void> {
  let seen = loaded.get(instance)
  if (!seen) {
    seen = new Set()
    loaded.set(instance, seen)
    registerUiCatalog(instance)
  }
  if (seen.has(lng)) return

  const bundles = await bundlesFor(lng)
  for (const [namespace, resources] of Object.entries(bundles)) {
    instance.addResourceBundle(lng, namespace, resources, true, true)
  }
  seen.add(lng)
}
