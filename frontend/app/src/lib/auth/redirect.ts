const FALLBACK = '/dashboard'

export function sanitizeReturnTo(raw: string | undefined | null): string {
  if (!raw || !raw.startsWith('/')) return FALLBACK
  // Reject protocol-relative URLs (//) and backslash escapes
  if (raw.startsWith('//') || raw.startsWith('/\\')) return FALLBACK
  return raw
}
