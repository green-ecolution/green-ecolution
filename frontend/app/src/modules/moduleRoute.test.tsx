import { describe, expect, it, vi } from 'vitest'
import { render, screen, waitFor } from '@testing-library/react'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import {
  createMemoryHistory,
  createRootRouteWithContext,
  createRoute,
  createRouter,
  RouterProvider,
} from '@tanstack/react-router'
import { http, HttpResponse } from 'msw'
import { server } from '@/test/mocks/server'
import type { ModuleManifest } from './types'

// The real session needs OIDC settings the test environment does not have.
vi.mock('@/lib/auth/session', () => ({
  getAuthSession: () => ({ getAccessToken: () => Promise.resolve(null) }),
}))

vi.mock('./registry', () => ({
  moduleManifests: [{ key: 'waste' }] as ModuleManifest[],
}))

const { moduleRoute } = await import('./moduleRoute')

const respondWith = (active: string[]) =>
  server.use(
    http.get('*/v1/modules', () =>
      HttpResponse.json({
        modules: ['waste'],
        organizations: [{ organizationId: 'a', active }],
      }),
    ),
  )

const renderRoute = (guardOptions: object = {}, cachedModules?: object) => {
  const queryClient = new QueryClient({ defaultOptions: { queries: { retry: false } } })
  if (cachedModules) queryClient.setQueryData(['modules'], cachedModules)
  const rootRoute = createRootRouteWithContext<{ queryClient: QueryClient }>()()
  const wasteRoute = createRoute({
    getParentRoute: () => rootRoute,
    path: '/waste',
    ...moduleRoute('waste', {
      component: () => <div>Waste content</div>,
      ...guardOptions,
    }),
  })
  const router = createRouter({
    routeTree: rootRoute.addChildren([wasteRoute]),
    history: createMemoryHistory({ initialEntries: ['/waste'] }),
    context: { queryClient },
  })
  render(
    <QueryClientProvider client={queryClient}>
      <RouterProvider router={router} />
    </QueryClientProvider>,
  )
  return queryClient
}

describe('moduleRoute', () => {
  it('shows the inactive notice instead of the route content', async () => {
    respondWith([])
    renderRoute()

    expect(await screen.findByRole('heading', { name: 'Modul nicht aktiv' })).toBeInTheDocument()
    expect(screen.queryByText('Waste content')).not.toBeInTheDocument()
  })

  it('renders the route when the module is active', async () => {
    respondWith(['waste'])
    renderRoute()

    expect(await screen.findByText('Waste content')).toBeInTheDocument()
  })

  it('refetches a stale modules answer so a lock reaches an open session', async () => {
    respondWith([])
    const queryClient = renderRoute(
      {},
      { modules: ['waste'], organizations: [{ organizationId: 'a', active: ['waste'] }] },
    )

    await waitFor(() =>
      expect(
        queryClient.getQueryData<{ organizations: { active: string[] }[] }>(['modules'])
          ?.organizations[0]?.active,
      ).toEqual([]),
    )
  })

  it('runs the wrapped beforeLoad after the activation check', async () => {
    respondWith(['waste'])
    const beforeLoad = vi.fn()
    renderRoute({ beforeLoad })

    expect(await screen.findByText('Waste content')).toBeInTheDocument()
    expect(beforeLoad).toHaveBeenCalledOnce()
  })
})
