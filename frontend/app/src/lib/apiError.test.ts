import { describe, it, expect } from 'vitest'
import { FetchError, ResponseError } from '@green-ecolution/backend-client'
import { resolveApiError } from './apiError'
import { apiErrorMessages } from './apiErrorMessages'

const respond = (status: number, body?: unknown, contentType = 'application/json') =>
  new ResponseError(
    new Response(body === undefined ? null : JSON.stringify(body), {
      status,
      headers: { 'content-type': contentType },
    }),
  )

describe('resolveApiError', () => {
  it('prefers the named cause over the status text', async () => {
    const info = await resolveApiError(
      respond(422, {
        error: 'the selected trees belong to a different organization than the cluster',
        code: 'organization_mismatch.trees_vs_cluster',
      }),
    )

    expect(info.status).toBe(422)
    expect(info.code).toBe('organization_mismatch.trees_vs_cluster')
    expect(info.messageKey).toBe('code.organization_mismatch.trees_vs_cluster')
    expect(info.message).toBe(apiErrorMessages['code.organization_mismatch.trees_vs_cluster'])
  })

  it('falls back to the status text for an unknown code', async () => {
    const info = await resolveApiError(
      respond(422, { error: 'something new', code: 'not_in_catalog' }),
    )

    expect(info.messageKey).toBe('status.422')
    expect(info.message).toBe(apiErrorMessages['status.422'])
    expect(info.code).toBe('not_in_catalog')
  })

  it('keeps the status text when the body names no cause', async () => {
    const info = await resolveApiError(respond(409, { error: 'resource already exists' }))

    expect(info.messageKey).toBe('status.409')
    expect(info.detail).toBe('resource already exists')
  })

  it('uses the backend prose when the status is unmapped', async () => {
    const info = await resolveApiError(respond(418, { error: 'I am a teapot' }))

    expect(info.message).toBe('I am a teapot')
    expect(info.messageKey).toBe('detail')
  })

  it('reports the status when neither catalog nor body help', async () => {
    const info = await resolveApiError(respond(418))

    expect(info.message).toContain('418')
    expect(info.messageKey).toBe('status.unknown')
  })

  it('survives a non-JSON body from a proxy', async () => {
    const error = new ResponseError(
      new Response('<html>502 Bad Gateway</html>', {
        status: 502,
        headers: { 'content-type': 'text/html' },
      }),
    )
    const info = await resolveApiError(error)

    expect(info.messageKey).toBe('status.502')
    expect(info.detail).toContain('Bad Gateway')
  })

  it('reports an unreachable server', async () => {
    const info = await resolveApiError(new FetchError(new Error('network down')))

    expect(info.messageKey).toBe('offline')
  })
})
