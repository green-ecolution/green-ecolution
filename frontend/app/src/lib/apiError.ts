import { FetchError, ResponseError } from '@green-ecolution/backend-client'
import { translateIssue, type ServerValidationIssue } from '@green-ecolution/domain-wasm'
import { isHTTPError } from './utils'
import { messageFor, type ApiErrorMessageKey } from './apiErrorMessages'

export interface ApiErrorInfo {
  /** German sentence intended for the user. */
  message: string
  /** Catalog key the message came from, so an i18n layer can re-translate it. */
  messageKey: ApiErrorMessageKey
  /** Stable cause the backend named, when it sent one. */
  code?: string
  /**
   * Which input field broke which rule, when the backend blamed one field.
   * Carries the same key as the in-browser validator, so `translateIssue`
   * renders it and a form can attach it to the field.
   */
  validation?: ServerValidationIssue
  /** Raw backend detail, kept for logging and unmapped causes. */
  detail?: string
  status?: number
}

interface ErrorPayload {
  detail?: string
  code?: string
  validation?: ServerValidationIssue
}

function readValidationIssue(parsed: object): ServerValidationIssue | undefined {
  if (!('validation' in parsed)) return undefined
  const issue: unknown = parsed.validation
  if (issue === null || typeof issue !== 'object') return undefined
  if (!('field' in issue) || !('key' in issue) || !('params' in issue)) return undefined
  if (typeof issue.field !== 'string' || typeof issue.key !== 'string') return undefined
  return issue as ServerValidationIssue
}

// Error bodies are JSON `{ error, code? }`, but a proxy or a crash can still
// deliver HTML or an empty body — parsing must never throw here, or the error
// handling itself becomes the reported error.
async function readPayload(response: Response): Promise<ErrorPayload> {
  try {
    const text = await response.clone().text()
    if (!text.trim()) return {}
    try {
      const parsed: unknown = JSON.parse(text)
      if (!isHTTPError(parsed)) return { detail: text }
      const code = 'code' in parsed && typeof parsed.code === 'string' ? parsed.code : undefined
      return { detail: parsed.error, code, validation: readValidationIssue(parsed) }
    } catch {
      return { detail: text }
    }
  } catch {
    return {}
  }
}

/**
 * The named cause wins over the status, because a status covers several causes
 * — a 422 alone cannot say whether an input was missing or two entities from
 * different organizations were combined.
 */
function resolveMessage(
  status: number,
  { detail, code, validation }: ErrorPayload,
): Pick<ApiErrorInfo, 'message' | 'messageKey'> {
  // A named field beats a generic code: "Name darf maximal 255 Zeichen lang
  // sein" is more use than "Die eingegebenen Daten sind ungültig."
  if (validation) {
    const message = translateIssue(validation)
    if (message !== validation.key) return { message, messageKey: `code.${validation.key}` }
  }
  const keys: ApiErrorMessageKey[] = code
    ? [`code.${code}`, `status.${status}`]
    : [`status.${status}`]
  for (const key of keys) {
    const message = messageFor(key)
    if (message) return { message, messageKey: key }
  }
  if (detail) return { message: detail, messageKey: 'detail' }
  return {
    message: messageFor('status.unknown', { status }) ?? `Status ${status}`,
    messageKey: 'status.unknown',
  }
}

export async function resolveApiError(error: unknown): Promise<ApiErrorInfo> {
  if (error instanceof ResponseError) {
    const status = error.response.status
    const payload = await readPayload(error.response)
    return {
      status,
      code: payload.code,
      validation: payload.validation,
      detail: payload.detail,
      ...resolveMessage(status, payload),
    }
  }

  if (error instanceof FetchError) {
    return { message: messageFor('offline') ?? '', messageKey: 'offline' }
  }

  if (error instanceof Error && error.message) {
    return { message: error.message, messageKey: 'detail' }
  }

  return { message: messageFor('unknown') ?? '', messageKey: 'unknown' }
}

/** Carries the user-facing message so React Query consumers can render it directly. */
export class ApiError extends Error {
  constructor(readonly info: ApiErrorInfo) {
    super(info.message)
    this.name = 'ApiError'
  }
}

export async function toApiError(error: unknown): Promise<ApiError> {
  return new ApiError(await resolveApiError(error))
}
