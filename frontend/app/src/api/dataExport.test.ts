import { describe, it, expect, vi } from 'vitest'
import { http, HttpResponse } from 'msw'
import { QueryClient } from '@tanstack/react-query'
import type { DataExportTargetResponse } from '@/api/backendApi'
import { server } from '@/test/mocks/server'
import {
  dataExportQueries,
  formValuesFrom,
  toDataExportRequest,
  type DataExportFormValues,
} from './dataExport'

vi.mock('@/lib/auth/session', () => ({
  getAuthSession: () => ({ getAccessToken: () => Promise.resolve(null) }),
}))

const blank: DataExportFormValues = {
  enabled: true,
  baseUrl: '  https://frost.example/v1.1  ',
  authKind: 'none',
  username: '',
  password: '',
  tokenUrl: '',
  clientId: '',
  clientSecret: '',
  scope: '',
}

describe('toDataExportRequest', () => {
  it('maps no authentication and trims the base url', () => {
    expect(toDataExportRequest(blank)).toEqual({
      enabled: true,
      baseUrl: 'https://frost.example/v1.1',
      auth: { kind: 'none' },
    })
  })

  it('omits an empty basic password', () => {
    const request = toDataExportRequest({ ...blank, authKind: 'basic', username: ' alice ' })
    expect(request.auth).toEqual({ kind: 'basic', username: 'alice' })
    expect('password' in request.auth).toBe(false)
  })

  it('sends a basic password when given', () => {
    const request = toDataExportRequest({
      ...blank,
      authKind: 'basic',
      username: 'alice',
      password: 'pw',
    })
    expect(request.auth).toEqual({ kind: 'basic', username: 'alice', password: 'pw' })
  })

  it('omits an empty client secret and a blank scope', () => {
    const request = toDataExportRequest({
      ...blank,
      authKind: 'oauth2_client_credentials',
      tokenUrl: ' https://auth.example/token ',
      clientId: ' client ',
      scope: '   ',
    })
    expect(request.auth).toEqual({
      kind: 'oauth2_client_credentials',
      tokenUrl: 'https://auth.example/token',
      clientId: 'client',
    })
    expect('clientSecret' in request.auth).toBe(false)
    expect('scope' in request.auth).toBe(false)
  })

  it('omits a whitespace-only password', () => {
    const request = toDataExportRequest({
      ...blank,
      authKind: 'basic',
      username: 'alice',
      password: '   ',
    })
    expect('password' in request.auth).toBe(false)
  })

  it('sends secret untrimmed and scope trimmed when given', () => {
    const request = toDataExportRequest({
      ...blank,
      authKind: 'oauth2_client_credentials',
      tokenUrl: 'https://auth.example/token',
      clientId: 'client',
      clientSecret: ' s3 ',
      scope: ' read ',
    })
    expect(request.auth).toEqual({
      kind: 'oauth2_client_credentials',
      tokenUrl: 'https://auth.example/token',
      clientId: 'client',
      clientSecret: ' s3 ',
      scope: 'read',
    })
  })
})

const response = (auth: DataExportTargetResponse['auth']): DataExportTargetResponse => ({
  activatedAt: new Date(),
  auth,
  baseUrl: 'https://frost.example/v1.1',
  enabled: false,
  kind: 'sensorthings',
  organizationId: 'org-1',
  status: {},
})

describe('formValuesFrom', () => {
  it('returns defaults without a target', () => {
    expect(formValuesFrom()).toEqual({
      enabled: true,
      baseUrl: '',
      authKind: 'none',
      username: '',
      password: '',
      tokenUrl: '',
      clientId: '',
      clientSecret: '',
      scope: '',
    })
  })

  it('maps a basic target without a password', () => {
    const values = formValuesFrom(response({ kind: 'basic', hasSecret: true, username: 'alice' }))
    expect(values).toMatchObject({
      enabled: false,
      baseUrl: 'https://frost.example/v1.1',
      authKind: 'basic',
      username: 'alice',
      password: '',
      clientSecret: '',
    })
  })

  it('maps an oauth target without a secret', () => {
    const values = formValuesFrom(
      response({
        kind: 'oauth2_client_credentials',
        hasSecret: true,
        clientId: 'client',
        tokenUrl: 'https://auth.example/token',
        scope: 'read',
      }),
    )
    expect(values).toMatchObject({
      authKind: 'oauth2_client_credentials',
      clientId: 'client',
      tokenUrl: 'https://auth.example/token',
      scope: 'read',
      password: '',
      clientSecret: '',
    })
  })
})

describe('dataExportQueries.byOrganization', () => {
  const run = () => new QueryClient().fetchQuery(dataExportQueries.byOrganization('org-1'))

  it('maps a 200 to configured', async () => {
    server.use(
      http.get('*/v1/organizations/org-1/data-export', () =>
        HttpResponse.json({
          activated_at: null,
          auth: { kind: 'none', has_secret: false },
          base_url: 'https://frost.example/v1.1',
          enabled: true,
          kind: 'sensorthings',
          organization_id: 'org-1',
          status: {},
        }),
      ),
    )
    const state = await run()
    expect(state.kind).toBe('configured')
    if (state.kind === 'configured') expect(state.target.baseUrl).toBe('https://frost.example/v1.1')
  })

  it('maps a 503 to disabled', async () => {
    server.use(
      http.get('*/v1/organizations/org-1/data-export', () =>
        HttpResponse.json({ error: 'off', code: 'feature.data_export_disabled' }, { status: 503 }),
      ),
    )
    expect(await run()).toEqual({ kind: 'disabled' })
  })

  it('maps a 404 to none', async () => {
    server.use(
      http.get('*/v1/organizations/org-1/data-export', () =>
        HttpResponse.json({ error: 'missing' }, { status: 404 }),
      ),
    )
    expect(await run()).toEqual({ kind: 'none' })
  })

  it('rethrows other errors', async () => {
    server.use(
      http.get('*/v1/organizations/org-1/data-export', () =>
        HttpResponse.json({ error: 'boom' }, { status: 500 }),
      ),
    )
    await expect(run()).rejects.toBeDefined()
  })
})
