import {
  FetchError,
  ResponseError,
  instanceOfErrorBody,
  instanceOfValidationIssue,
  type ValidationIssue,
} from '@green-ecolution/backend-client'
import { translateIssueNow } from '@/lib/i18n/validation'
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
  validation?: ValidationIssue
  /** Raw backend detail, kept for logging and unmapped causes. */
  detail?: string
  status?: number
}

interface ErrorPayload {
  detail?: string
  code?: string
  validation?: ValidationIssue
}

// The generated guards only assert that the required keys exist, which is
// exactly the check that matters here: the body crossed the network and may be
// anything.
function readValidationIssue(issue: unknown): ValidationIssue | undefined {
  if (issue === null || typeof issue !== 'object') return undefined
  return instanceOfValidationIssue(issue) ? issue : undefined
}

// Error bodies follow the generated `ErrorBody` schema, but a proxy or a crash
// can still deliver HTML or an empty body — parsing must never throw here, or
// the error handling itself becomes the reported error.
async function readPayload(response: Response): Promise<ErrorPayload> {
  try {
    const text = await response.clone().text()
    if (!text.trim()) return {}
    try {
      const parsed: unknown = JSON.parse(text)
      if (parsed === null || typeof parsed !== 'object') return { detail: text }
      if (!instanceOfErrorBody(parsed)) return { detail: text }
      return {
        detail: parsed.error,
        code: parsed.code ?? undefined,
        validation: readValidationIssue(parsed.validation),
      }
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
    const message = translateIssueNow(validation)
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
