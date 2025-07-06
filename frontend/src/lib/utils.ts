import { HTTPError } from '@green-ecolution/backend-client'
import { z } from 'zod'

export function decodeJWT<T>(token: string): T {
  const payload = token.split('.')[1]
  const base64 = payload.replace(/-/g, '+').replace(/_/g, '/')

  // Decode Base64 to string
  const jsonString = decodeURIComponent(
    atob(base64)
      .split('')
      .map((c) => '%' + ('00' + c.charCodeAt(0).toString(16)).slice(-2))
      .join(''),
  )

  return JSON.parse(jsonString) as T
}

export function roundTo(n: number, digits: number) {
  return Number(Math.round(Number(n + 'e' + digits)) + 'e-' + digits)
}

export function isHTTPError(data: unknown): data is HTTPError {
  return (
    typeof data === 'object' && data !== null && 'error' in data && typeof data.error === 'string'
  )
}

type ParseResult<T> =
  | { data: T; success: true; error?: undefined }
  | { data?: undefined; success: false; error?: unknown }

export const safeJsonStorageParse = <T>(
  key: string,
  opts: { storage?: Storage; schema: z.ZodSchema<T> },
): ParseResult<T> => {
  const storage = opts.storage ?? window.sessionStorage
  const text = storage.getItem(key)

  if (!text) {
    return { success: false }
  }

  try {
    const { success, data, error } = opts.schema.safeParse(JSON.parse(text))
    if (success) {
      return { data: data, success: true }
    } else {
      return { success: false, error: error }
    }
  } catch (err) {
    return { success: false, error: err }
  }
}
