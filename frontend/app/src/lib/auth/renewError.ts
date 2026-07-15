import { ErrorResponse } from 'oidc-client-ts'

// Only invalid_grant is fatal; network failures must not force a re-login (offline PWA)
export function isSessionDeadError(err: unknown): boolean {
  return err instanceof ErrorResponse && err.error === 'invalid_grant'
}
