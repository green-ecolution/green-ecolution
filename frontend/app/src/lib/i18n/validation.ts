import {
  translateIssue,
  type IssueTranslator,
  type TranslatableIssue,
} from '@green-ecolution/domain-wasm'
import { useCallback } from 'react'
import { useTranslation } from 'react-i18next'
import { getI18n } from './index'

// `t`'s generated overloads only accept the catalog's literal key union; a key
// coming from the Rust validator at runtime can't satisfy that statically.
type LooseTranslate = (key: string, params?: Record<string, string | number>) => string

/** For components and form hooks: re-renders when the language changes. */
export function useIssueTranslator(): IssueTranslator {
  const { t, i18n } = useTranslation('validation')
  return useCallback(
    // The key may not exist in the catalog (an unmapped Rust variant, or a key
    // the server names for a field the browser validator never covered).
    // i18n's `missingKeyHandler` throws for a key missing in every language,
    // so check first and let `translateIssue`'s own key fallback handle it.
    (key, params) =>
      i18n.exists(key, { ns: 'validation' }) ? (t as LooseTranslate)(key, params) : '',
    [t, i18n],
  )
}

/** For non-React callers such as `resolveApiError`. */
export function translateIssueNow(issue: TranslatableIssue): string {
  const i18n = getI18n()
  return translateIssue(issue, (key, params) => {
    const namespaced = `validation:${key}`
    return i18n.exists(namespaced) ? (i18n.t as LooseTranslate)(namespaced, params) : ''
  })
}
