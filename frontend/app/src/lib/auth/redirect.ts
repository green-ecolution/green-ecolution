const FALLBACK = '/dashboard'

export function sanitizeReturnTo(raw: string | undefined | null): string {
  if (!raw?.startsWith('/')) return FALLBACK
  // //, /\\, and https:// all slip past the startsWith('/') check — open-redirect vectors
  if (raw.startsWith('//') || raw.startsWith('/\\')) return FALLBACK
  return raw
}
