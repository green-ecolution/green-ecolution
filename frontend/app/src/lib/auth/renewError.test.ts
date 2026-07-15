import { describe, it, expect } from 'vitest'
import { ErrorResponse } from 'oidc-client-ts'
import { isSessionDeadError } from './renewError'

describe('isSessionDeadError', () => {
  it('is true for invalid_grant (Keycloak revoked/expired the session)', () => {
    const err = new ErrorResponse({ error: 'invalid_grant' })
    expect(isSessionDeadError(err)).toBe(true)
  })

  it('is false for other OIDC errors', () => {
    const err = new ErrorResponse({ error: 'server_error' })
    expect(isSessionDeadError(err)).toBe(false)
  })

  it('is false for network failures', () => {
    expect(isSessionDeadError(new TypeError('Failed to fetch'))).toBe(false)
  })
})
