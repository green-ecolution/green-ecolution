import {
  translateIssue,
  type IssueTranslator,
  type TranslatableIssue,
} from '@green-ecolution/domain-wasm'
import { useCallback } from 'react'
import { useTranslation } from 'react-i18next'
import { moduleManifests } from '@/modules/registry'
import { getI18n } from './index'

// `t`'s generated overloads only accept the catalog's literal key union; a key
// coming from the Rust validator at runtime can't satisfy that statically.
type LooseTranslate = (key: string, params?: Record<string, string | number>) => string

/**
 * Where the text of a validation key may be found, in lookup order. A module
 * keeps the keys of its own objects under `<namespace>:validation.*` and may
 * word a core key for itself there. The module the form or request belongs to
 * goes first, then the core catalog, then every other module.
 */
export function validationCatalogKeys(key: string, module?: string): string[] {
  const inModules = (keep: (moduleKey: string) => boolean) =>
    moduleManifests
      .filter(({ key: moduleKey }) => keep(moduleKey))
      .flatMap(({ i18n }) => i18n.namespaces.map((namespace) => `${namespace}:validation.${key}`))
  return [
    ...inModules((moduleKey) => moduleKey === module),
    `validation:${key}`,
    ...inModules((moduleKey) => moduleKey !== module),
  ]
}

/** For components and form hooks: re-renders when the language changes. */
export function useIssueTranslator(module?: string): IssueTranslator {
  const { t, i18n } = useTranslation('validation')
  return useCallback(
    // The key may not exist in any catalog (an unmapped Rust variant, or a key
    // the server names for a field the browser validator never covered).
    // i18n's `missingKeyHandler` throws for a key missing in every language,
    // so check first and let `translateIssue`'s own key fallback handle it.
    (key, params) => {
      const found = validationCatalogKeys(key, module).find((candidate) => i18n.exists(candidate))
      return found ? (t as LooseTranslate)(found, params) : ''
    },
    [t, i18n, module],
  )
}

/** For non-React callers such as `resolveApiError`. */
export function translateIssueNow(issue: TranslatableIssue, module?: string): string {
  const i18n = getI18n()
  return translateIssue(issue, (key, params) => {
    const found = validationCatalogKeys(key, module).find((candidate) => i18n.exists(candidate))
    return found ? (i18n.t as LooseTranslate)(found, params) : ''
  })
}
