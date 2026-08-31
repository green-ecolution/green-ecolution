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

export async function ensureLanguage(instance: I18n, lng: Language): Promise<void> {
  let seen = loaded.get(instance)
  if (!seen) {
    seen = new Set()
    loaded.set(instance, seen)
  }
  if (seen.has(lng)) return

  const bundles = await bundlesFor(lng)
  for (const [namespace, resources] of Object.entries(bundles)) {
    instance.addResourceBundle(lng, namespace, resources, true, true)
  }
  seen.add(lng)
}
