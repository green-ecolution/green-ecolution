import type {
  FieldError,
  FieldErrors,
  FieldValues,
  Resolver,
  ResolverResult,
} from 'react-hook-form'
import { translateIssue } from './messages'
import type { IssueTranslator, ValidationIssue } from './types'

// JS-side normalisation before crossing the WASM boundary. Domain DTOs use
// strict serde types (RFC 3339 strings for dates), but RHF stores `Date`
// objects. Strict deserialization would throw before validation runs, so we
// shape the values into wire-format types here.
function normaliseForWasm(values: unknown): unknown {
  if (values instanceof Date) return values.toISOString()
  if (Array.isArray(values)) return values.map(normaliseForWasm)
  if (values !== null && typeof values === 'object') {
    const out: Record<string, unknown> = {}
    for (const [k, v] of Object.entries(values)) {
      out[k] = normaliseForWasm(v)
    }
    return out
  }
  return values
}

/**
 * Turns a draft validator of the WASM build into a React Hook Form resolver.
 * wasm-bindgen types those validators as `(input: any) => any`; what they
 * return is a list of issues.
 */
export function draftResolver<TForm extends FieldValues>(
  validate: (input: unknown) => unknown,
  translate: IssueTranslator,
): Resolver<TForm> {
  return async (values): Promise<ResolverResult<TForm>> => {
    const issues = validate(normaliseForWasm(values)) as ValidationIssue[]
    if (issues.length === 0) {
      return { values, errors: {} }
    }
    const errors: Record<string, FieldError> = {}
    for (const issue of issues) {
      errors[issue.path] = {
        type: issue.key,
        message: translateIssue(issue, translate),
      }
    }
    // RHF's FieldErrors<TForm> uses statically-keyed types; runtime issue.path
    // is dynamic, so the structurally-equivalent record needs one boundary cast.
    return { values: {}, errors: errors as FieldErrors<TForm> }
  }
}
