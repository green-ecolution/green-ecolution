import { useTranslation } from 'react-i18next'
import type { ParseKeys } from 'i18next'
import { NAMESPACES } from './languages'

/**
 * Mirrors `Breadcrumb`'s `{ titleKey } | { title }` union (Task 11): route
 * modules (`routeTree.gen.ts`) are imported and evaluated before
 * `createI18n()` resolves, so a translated label reaching a route option must
 * be a key, not a pre-resolved string — and it must stay a key rather than
 * being resolved once at declaration time, or it goes stale if the language
 * changes (e.g. via `?lng=`) while the owning component stays mounted.
 * `options` carries i18next interpolation values for labels that need them.
 *
 * `ParseKeys<typeof NAMESPACES>` is the same cross-namespace key union
 * `i18n.t` itself accepts (`NAMESPACES[0]` is `'common'`, the default
 * namespace, exactly like `i18n.t`'s own `[...DefaultNS, ...OtherNS]` typing),
 * so `{ key: 'tree:...' }` type-checks the same way `getI18n().t('tree:...')`
 * already does elsewhere — passing a plain `FlatNamespace` union here instead
 * of the ordered tuple silently degrades to unprefixed keys only, which is
 * why this imports the tuple rather than deriving one ad hoc.
 */
export type LocalizedText =
  string | { key: ParseKeys<typeof NAMESPACES>; options?: Record<string, unknown> }

/**
 * Resolves a `LocalizedText` reactively. `i18n` here is react-i18next's own
 * per-render wrapper around the instance (see `useTranslation`'s
 * implementation): it subscribes this component to `languageChanged` and is
 * recreated whenever the language flips, so callers re-render and re-resolve
 * automatically — no separate subscription needed.
 */
export function useLocalizedText(): (value: LocalizedText) => string {
  const { i18n } = useTranslation()
  return (value) => (typeof value === 'string' ? value : i18n.t(value.key, value.options))
}
