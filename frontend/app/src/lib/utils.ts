import { getI18n } from '@/lib/i18n'
import { intlLocale } from '@/lib/i18n/format'

interface HTTPError {
  error: string
}

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

export function formatKm(meters: number): string {
  const locale = intlLocale(getI18n().language)
  return `${new Intl.NumberFormat(locale, { maximumFractionDigits: 2 }).format(meters / 1000)} km`
}

export function isHTTPError(data: unknown): data is HTTPError {
  return (
    typeof data === 'object' && data !== null && 'error' in data && typeof data.error === 'string'
  )
}

export function formatLiters(liters: number): string {
  const locale = intlLocale(getI18n().language)
  return `${new Intl.NumberFormat(locale, { maximumFractionDigits: 0 }).format(liters)} L`
}

// RHF `setValueAs` for number inputs: German keyboards produce a decimal
// comma, and unparsable input must stay a string so the domain validator
// reports it as a field error instead of the value silently reaching the API.
export function parseDecimalInput(value: unknown): unknown {
  if (typeof value !== 'string') return value
  const trimmed = value.trim()
  if (trimmed === '') return value
  const parsed = Number(trimmed.replace(',', '.'))
  return Number.isFinite(parsed) ? parsed : value
}
