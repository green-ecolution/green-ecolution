export interface ValidationIssue {
  path: string
  field: string
  key: string
  params: Record<string, string | number>
}

/** The minimum needed to render an issue: which rule broke, with what values. */
export type TranslatableIssue = Pick<ValidationIssue, 'key' | 'params'>

/**
 * Renders one validation key. Injected by the consumer, because this package
 * must not depend on a translation library and must not carry UI copy.
 */
export type IssueTranslator = (key: string, params: Record<string, string | number>) => string
