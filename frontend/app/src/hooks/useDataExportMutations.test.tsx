import { describe, it, expect, vi, beforeEach } from 'vitest'
import { act } from '@testing-library/react'
import { http, HttpResponse } from 'msw'
import { server } from '@/test/mocks/server'
import { renderHookWithClient } from '@/test/utils'
import type { DataExportFormValues } from '@/api/dataExport'
import { useDataExportMutations } from './useDataExportMutations'

vi.mock('@/lib/auth/session', () => ({
  getAuthSession: () => ({ getAccessToken: () => Promise.resolve(null) }),
}))

const showToast = vi.fn()
vi.mock('@/hooks/createToast', () => ({ default: () => showToast }))

const values: DataExportFormValues = {
  enabled: true,
  baseUrl: 'https://frost.example/v1.1',
  authKind: 'basic',
  username: 'alice',
  password: 'pw',
  tokenUrl: '',
  clientId: '',
  clientSecret: '',
  scope: '',
}

const targetBody = {
  activated_at: null,
  auth: { kind: 'basic', has_secret: true, username: 'alice' },
  base_url: 'https://frost.example/v1.1',
  enabled: true,
  kind: 'sensorthings',
  organization_id: 'org-1',
  status: {},
}

describe('useDataExportMutations', () => {
  beforeEach(() => vi.clearAllMocks())

  it('saves the mapped request and refreshes the target', async () => {
    let body: unknown
    server.use(
      http.put('*/v1/organizations/org-1/data-export', async ({ request }) => {
        body = await request.json()
        return HttpResponse.json(targetBody)
      }),
    )
    const { result, queryClient } = renderHookWithClient(() => useDataExportMutations('org-1'))
    const invalidate = vi.spyOn(queryClient, 'invalidateQueries')

    await act(() => result.current.save.mutateAsync(values))

    expect(body).toEqual({
      enabled: true,
      base_url: 'https://frost.example/v1.1',
      auth: { kind: 'basic', username: 'alice', password: 'pw' },
    })
    expect(invalidate).toHaveBeenCalledWith({ queryKey: ['organizations', 'org-1', 'data-export'] })
    expect(showToast).toHaveBeenCalledWith('Datenplattform gespeichert')
  })

  it('sends the oauth fields through the generated client', async () => {
    let body: unknown
    server.use(
      http.put('*/v1/organizations/org-1/data-export', async ({ request }) => {
        body = await request.json()
        return HttpResponse.json(targetBody)
      }),
    )
    const { result } = renderHookWithClient(() => useDataExportMutations('org-1'))

    await act(() =>
      result.current.save.mutateAsync({
        ...values,
        authKind: 'oauth2_client_credentials',
        tokenUrl: 'https://auth.example/token',
        clientId: 'client',
        clientSecret: 's3',
        scope: 'read',
      }),
    )

    expect(body).toEqual({
      enabled: true,
      base_url: 'https://frost.example/v1.1',
      auth: {
        kind: 'oauth2_client_credentials',
        token_url: 'https://auth.example/token',
        client_id: 'client',
        client_secret: 's3',
        scope: 'read',
      },
    })
  })

  it('removes the target and refreshes', async () => {
    let called = false
    server.use(
      http.delete('*/v1/organizations/org-1/data-export', () => {
        called = true
        return new HttpResponse(null, { status: 204 })
      }),
    )
    const { result, queryClient } = renderHookWithClient(() => useDataExportMutations('org-1'))
    const invalidate = vi.spyOn(queryClient, 'invalidateQueries')

    await act(() => result.current.remove.mutateAsync())

    expect(called).toBe(true)
    expect(invalidate).toHaveBeenCalledWith({ queryKey: ['organizations', 'org-1', 'data-export'] })
    expect(showToast).toHaveBeenCalledWith('Datenplattform entfernt')
  })

  it('returns the connection test without a toast', async () => {
    server.use(
      http.post('*/v1/organizations/org-1/data-export/test', () =>
        HttpResponse.json({ data_array_supported: true, outcome: 'ok', http_status: 200 }),
      ),
    )
    const { result } = renderHookWithClient(() => useDataExportMutations('org-1'))

    const outcome = await act(() => result.current.test.mutateAsync())

    expect(outcome.outcome).toBe('ok')
    expect(showToast).not.toHaveBeenCalled()
  })

  it('leaves a failed save to the caller without a toast', async () => {
    server.use(
      http.put('*/v1/organizations/org-1/data-export', () =>
        HttpResponse.json({ error: 'nope', code: 'data_export.url_not_allowed' }, { status: 422 }),
      ),
    )
    const { result } = renderHookWithClient(() => useDataExportMutations('org-1'))

    await expect(act(() => result.current.save.mutateAsync(values))).rejects.toBeDefined()

    await new Promise((resolve) => setTimeout(resolve, 0))
    expect(showToast).not.toHaveBeenCalled()
  })

  it('shows a failed removal as an error toast', async () => {
    server.use(
      http.delete('*/v1/organizations/org-1/data-export', () =>
        HttpResponse.json({ error: 'forbidden' }, { status: 403 }),
      ),
    )
    const { result } = renderHookWithClient(() => useDataExportMutations('org-1'))

    await act(() => result.current.remove.mutateAsync().catch(() => undefined))

    await vi.waitFor(() =>
      expect(showToast).toHaveBeenCalledWith(
        'Für diese Aktion fehlt dir die Berechtigung.',
        'error',
      ),
    )
  })
})
