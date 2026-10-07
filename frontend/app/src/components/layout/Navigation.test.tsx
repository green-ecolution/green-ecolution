/* eslint-disable @typescript-eslint/ban-ts-comment */
// @ts-nocheck - ad-hoc routes are not part of the generated route tree
import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen, waitFor } from '@testing-library/react'
import { http, HttpResponse } from 'msw'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { server } from '@/test/mocks/server'
import {
  createRootRoute,
  createRoute,
  createRouter,
  createMemoryHistory,
  RouterProvider,
  Outlet,
} from '@tanstack/react-router'
import { UNRESTRICTED, type Permissions } from '@/lib/auth/permissions'

const permissions = vi.fn((): Permissions => new Set<string>())

vi.mock('@/lib/auth/usePermissions', () => ({
  usePermissions: () => permissions(),
}))

vi.mock('@/lib/auth/authSessionContext', () => ({
  useAuthSession: () => ({ isAuthenticated: true, accessToken: null }),
}))

// The real session needs OIDC settings the test environment does not have.
vi.mock('@/lib/auth/session', () => ({
  getAuthSession: () => ({ getAccessToken: () => Promise.resolve(null) }),
}))

// The avatar hook would fire an unmocked /users/me request, which MSW rejects.
vi.mock('@/lib/auth/useCurrentUserAvatar', () => ({
  useCurrentUserAvatar: () => undefined,
}))

const { default: Navigation } = await import('./Navigation')

const renderNavigation = () => {
  const rootRoute = createRootRoute({ component: () => <Outlet /> })
  const indexRoute = createRoute({
    getParentRoute: () => rootRoute,
    path: '/',
    component: () => <Navigation isOpen closeSidebar={() => undefined} />,
  })
  const pluginRoute = createRoute({
    getParentRoute: () => rootRoute,
    path: '/plugin/$slug',
    component: () => null,
  })
  const router = createRouter({
    routeTree: rootRoute.addChildren([indexRoute, pluginRoute]),
    history: createMemoryHistory({ initialEntries: ['/'] }),
  })

  const queryClient = new QueryClient({ defaultOptions: { queries: { retry: false } } })
  return render(
    <QueryClientProvider client={queryClient}>
      <RouterProvider router={router} />
    </QueryClientProvider>,
  )
}

describe('Navigation', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  it('shows only the entries the permissions allow', async () => {
    permissions.mockReturnValue(new Set(['tree:read']))

    renderNavigation()

    await waitFor(() => {
      expect(screen.getByText('Bäume')).toBeInTheDocument()
    })
    // tree:read satisfies the map (tree OR cluster) and evaluations (any read).
    expect(screen.getByText('Karte')).toBeInTheDocument()
    expect(screen.getByText('Auswertung')).toBeInTheDocument()
    expect(screen.queryByText('Bewässerungsgruppen')).not.toBeInTheDocument()
    expect(screen.queryByText('Einsatzpläne')).not.toBeInTheDocument()
    expect(screen.queryByText('Fahrzeuge')).not.toBeInTheDocument()
    expect(screen.queryByText('Sensoren')).not.toBeInTheDocument()
  })

  it('drops the headline of a section without any reachable entry', async () => {
    permissions.mockReturnValue(new Set(['tree:read']))

    renderNavigation()

    await waitFor(() => {
      expect(screen.getByText('Grünflächen')).toBeInTheDocument()
    })
    expect(screen.queryByText('Planung')).not.toBeInTheDocument()
  })

  it('shows every entry for unrestricted access', async () => {
    permissions.mockReturnValue(UNRESTRICTED)

    renderNavigation()

    await waitFor(() => {
      expect(screen.getByText('Fahrzeuge')).toBeInTheDocument()
    })
    expect(screen.getByText('Bewässerungsgruppen')).toBeInTheDocument()
    expect(screen.getByText('Sensoren')).toBeInTheDocument()
    expect(screen.getByText('Einsatzpläne')).toBeInTheDocument()
    // Mitarbeitende lives under settings now, not the global nav.
    expect(screen.queryByText('Mitarbeitende')).not.toBeInTheDocument()
  })

  it('keeps the always-open settings entry for a user without any grant', async () => {
    permissions.mockReturnValue(new Set<string>())

    renderNavigation()

    await waitFor(() => {
      expect(screen.getByText('Einstellungen')).toBeInTheDocument()
    })
    expect(screen.queryByText('Grünflächen')).not.toBeInTheDocument()
    expect(screen.queryByText('Bäume')).not.toBeInTheDocument()
  })

  it('lists the plugin views the user may open', async () => {
    permissions.mockReturnValue(UNRESTRICTED)
    server.use(
      http.get('/api-local/v1/plugins/views', () =>
        HttpResponse.json([{ slug: 'sensor-setup', name: 'Sensor-Einrichtung' }]),
      ),
    )
    renderNavigation()

    expect(await screen.findByText('Plugins')).toBeVisible()
    expect(screen.getByRole('link', { name: 'Sensor-Einrichtung' })).toHaveAttribute(
      'href',
      '/plugin/sensor-setup',
    )
  })

  it('shows no plugin section when no view is openable', async () => {
    permissions.mockReturnValue(UNRESTRICTED)
    renderNavigation()

    await screen.findByText('Sensoren')
    expect(screen.queryByText('Plugins')).not.toBeInTheDocument()
  })

  it('shows no plugin section when the plugin feature is off', async () => {
    permissions.mockReturnValue(UNRESTRICTED)
    server.use(
      http.get('/api-local/v1/plugins/views', () =>
        HttpResponse.json({ error: 'disabled', code: 'feature.plugins_disabled' }, { status: 503 }),
      ),
    )
    renderNavigation()

    await screen.findByText('Sensoren')
    expect(screen.queryByText('Plugins')).not.toBeInTheDocument()
  })
})
