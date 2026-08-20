import { FetchError, ResponseError } from '@green-ecolution/backend-client'
import { isHTTPError } from './utils'

export interface ApiErrorInfo {
  /** German sentence intended for the user. */
  message: string
  /** Raw backend detail, kept for logging and unmapped status codes. */
  detail?: string
  status?: number
}

const statusMessages: Record<number, string> = {
  400: 'Die Anfrage war fehlerhaft und konnte nicht verarbeitet werden.',
  401: 'Die Anmeldung ist abgelaufen. Bitte melde dich erneut an.',
  403: 'Für diese Aktion fehlt dir die Berechtigung.',
  404: 'Der angeforderte Datensatz wurde nicht gefunden.',
  409: 'Der Datensatz existiert bereits oder wurde zwischenzeitlich geändert.',
  415: 'Das Format der Anfrage wird nicht unterstützt.',
  422: 'Die eingegebenen Daten sind unvollständig oder ungültig.',
  500: 'Auf dem Server ist ein unerwarteter Fehler aufgetreten.',
  502: 'Ein nachgelagerter Dienst hat nicht geantwortet.',
  503: 'Der Dienst ist derzeit nicht erreichbar. Bitte versuche es später erneut.',
}

// Error bodies are JSON `{ error }`, but a proxy or a crash can still deliver
// HTML or an empty body — parsing must never throw here, or the error handling
// itself becomes the reported error.
async function readDetail(response: Response): Promise<string | undefined> {
  try {
    const text = await response.clone().text()
    if (!text.trim()) return undefined
    try {
      const parsed: unknown = JSON.parse(text)
      return isHTTPError(parsed) ? parsed.error : text
    } catch {
      return text
    }
  } catch {
    return undefined
  }
}

export async function resolveApiError(error: unknown): Promise<ApiErrorInfo> {
  if (error instanceof ResponseError) {
    const status = error.response.status
    const detail = await readDetail(error.response)
    const mapped = statusMessages[status]
    return {
      status,
      detail,
      message: mapped ?? detail ?? `Der Server hat mit Status ${status} geantwortet.`,
    }
  }

  if (error instanceof FetchError) {
    return { message: 'Der Server ist nicht erreichbar. Bitte prüfe deine Verbindung.' }
  }

  if (error instanceof Error && error.message) {
    return { message: error.message }
  }

  return { message: 'Unbekannter Fehler' }
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
