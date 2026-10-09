import { describe, it, expect, vi, beforeEach } from 'vitest'
import { renderHook, waitFor } from '@testing-library/react'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import type { ReactNode } from 'react'
import { ResponseError } from '@green-ecolution/backend-client'

const installPlugin = vi.fn()
const updatePlugin = vi.fn()

vi.mock('@/api/backendApi', () => ({
  pluginApi: {
    installPlugin: (...args: unknown[]) => installPlugin(...args) as unknown,
    updatePlugin: (...args: unknown[]) => updatePlugin(...args) as unknown,
  },
}))

const showToast = vi.fn()
vi.mock('@/hooks/createToast', () => ({ default: () => showToast }))

const { usePluginMutations } = await import('./usePluginMutations')

const renderMutations = () => {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
  })
  const wrapper = ({ children }: { children: ReactNode }) => (
    <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
  )
  return renderHook(() => usePluginMutations(), { wrapper })
}

const coded = (code: string) =>
  new ResponseError(
    new Response(JSON.stringify({ error: 'rejected', code }), {
      status: 400,
      headers: { 'Content-Type': 'application/json' },
    }),
  )

const PROXY_UNAVAILABLE =
  'Der interne Proxy für Plugins ist in dieser Installation nicht eingerichtet.'
const TARGET_NOT_ALLOWED = 'Dieser Dienst steht nicht auf der Freigabeliste für Plugins.'

const createRequest = {
  slug: 'acme',
  name: 'Acme',
  organizationId: 'org-1',
  permissions: [],
  requiredPermissions: [],
  frontend: { mode: 'proxied' as const, target: 'kataster.plugins.svc:8080' },
}

describe('usePluginMutations', () => {
  beforeEach(() => vi.clearAllMocks())

  describe('install', () => {
    it.each([
      ['plugin.proxy_unavailable', PROXY_UNAVAILABLE],
      ['plugin.proxy_target_not_allowed', TARGET_NOT_ALLOWED],
    ])('shows the translated message for %s', async (code, message) => {
      installPlugin.mockRejectedValue(coded(code))
      const { result } = renderMutations()

      result.current.installPlugin.mutate(createRequest)

      await waitFor(() => expect(showToast).toHaveBeenCalledWith(message, 'error'))
    })

    it('falls back to the generic message for an uncoded error', async () => {
      installPlugin.mockRejectedValue(new ResponseError(new Response(null, { status: 500 })))
      const { result } = renderMutations()

      result.current.installPlugin.mutate(createRequest)

      await waitFor(() =>
        expect(showToast).toHaveBeenCalledWith(
          'Das Plugin konnte nicht installiert werden.',
          'error',
        ),
      )
    })
  })

  describe('update', () => {
    it.each([
      ['plugin.proxy_unavailable', PROXY_UNAVAILABLE],
      ['plugin.proxy_target_not_allowed', TARGET_NOT_ALLOWED],
    ])('shows the translated message for %s', async (code, message) => {
      updatePlugin.mockRejectedValue(coded(code))
      const { result } = renderMutations()

      result.current.updatePlugin.mutate({ slug: 'acme', change: { name: 'Acme' } })

      await waitFor(() => expect(showToast).toHaveBeenCalledWith(message, 'error'))
    })

    it('falls back to the generic message for an uncoded error', async () => {
      updatePlugin.mockRejectedValue(new ResponseError(new Response(null, { status: 500 })))
      const { result } = renderMutations()

      result.current.updatePlugin.mutate({ slug: 'acme', change: { name: 'Acme' } })

      await waitFor(() =>
        expect(showToast).toHaveBeenCalledWith(
          'Das Plugin konnte nicht gespeichert werden.',
          'error',
        ),
      )
    })
  })
})
