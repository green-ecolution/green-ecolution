// RFC 7636 — PKCE for OAuth 2.0 public clients.
// Verifier: 43–128 chars from [A-Z][a-z][0-9]-._~. We generate 32 random bytes
// → 43 base64url chars. Challenge: base64url(sha256(verifier)).

const VERIFIER_STORAGE_KEY = 'pkce_verifier'

function base64UrlEncode(bytes: Uint8Array): string {
  let binary = ''
  for (const byte of bytes) {
    binary += String.fromCharCode(byte)
  }
  return btoa(binary).replace(/\+/g, '-').replace(/\//g, '_').replace(/=+$/, '')
}

export function generateCodeVerifier(): string {
  const bytes = new Uint8Array(32)
  crypto.getRandomValues(bytes)
  return base64UrlEncode(bytes)
}

export async function deriveCodeChallenge(verifier: string): Promise<string> {
  const hash = await crypto.subtle.digest('SHA-256', new TextEncoder().encode(verifier))
  return base64UrlEncode(new Uint8Array(hash))
}

export function storeVerifier(verifier: string): void {
  sessionStorage.setItem(VERIFIER_STORAGE_KEY, verifier)
}

export function takeVerifier(): string | null {
  const verifier = sessionStorage.getItem(VERIFIER_STORAGE_KEY)
  if (verifier) {
    sessionStorage.removeItem(VERIFIER_STORAGE_KEY)
  }
  return verifier
}
