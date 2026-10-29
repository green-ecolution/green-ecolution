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

const adminMode = vi.hoisted(() => ({ value: false }))
vi.mock('@/lib/auth/scope/adminMode', () => ({
  useAdminMode: () => adminMode.value,
}))

vi.mock('@/modules/registry', async () => {
  const actual = await vi.importActual<typeof import('@/modules/registry')>('@/modules/registry')
  const probe = {
    key: 'probe',
    labelKey: 'irrigation:label',
    i18n: { namespaces: [], load: () => Promise.resolve({}) },
    nav: {
      links: [
        {
          key: 'nav-probe',
          to: '/probe',
          labelKey: 'irrigation:label',
          icon: () => null,
          section: 'planning',
        },
      ],
    },
  }
  return { moduleManifests: [...actual.moduleManifests, probe] }
})

const { default: Navigation } = await import('./Navigation')

const respondWithActive = (active: string[]) =>
  server.use(
    http.get('*/v1/modules', () =>
      HttpResponse.json({
        modules: ['irrigation', 'probe'],
        organizations: [{ organizationId: 'a', active }],
      }),
    ),
  )

const respondWithServices = (pluginsEnabled: boolean) =>
  server.use(
    http.get('*/v1/info/services', () =>
      HttpResponse.json({
        items: [
          {
            name: 'plugins',
            enabled: pluginsEnabled,
            healthy: true,
            lastChecked: '2026-10-28T00:00:00Z',
            message: '',
            responseTimeMs: 0,
          },
        ],
      }),
    ),
  )

const settledPluginViews = (queryClient: QueryClient) =>
  waitFor(() => {
    const state = queryClient.getQueryState(['plugins', 'views'])
    expect(state?.status === 'success' || state?.status === 'error').toBe(true)
  })

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
  const result = render(
    <QueryClientProvider client={queryClient}>
      <RouterProvider router={router} />
    </QueryClientProvider>,
  )
  return { ...result, queryClient }
}

describe('Navigation', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    adminMode.value = false
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
      expect(screen.getByText('Bewässerungsgruppen')).toBeInTheDocument()
    })
    expect(screen.getByText('Fahrzeuge')).toBeInTheDocument()
    expect(screen.getByText('Bewässerungsgruppen')).toBeInTheDocument()
    expect(screen.getByText('Sensoren')).toBeInTheDocument()
    expect(screen.getByText('Einsatzpläne')).toBeInTheDocument()
    // Mitarbeitende lives under settings now, not the global nav.
    expect(screen.queryByText('Mitarbeitende')).not.toBeInTheDocument()
  })

  it('lists the link of a module only while the module is active', async () => {
    permissions.mockReturnValue(UNRESTRICTED)
    respondWithActive(['probe'])
    renderNavigation()

    expect(await screen.findByText('Bewässerung')).toBeInTheDocument()
  })

  it('omits the link of a module that is not active', async () => {
    permissions.mockReturnValue(UNRESTRICTED)
    respondWithActive(['irrigation'])
    renderNavigation()

    await screen.findByText('Bäume')
    expect(screen.queryByText('Bewässerung')).not.toBeInTheDocument()
  })

  it('shows neither map, trees, clusters, evaluations nor plans without an active module', async () => {
    permissions.mockReturnValue(UNRESTRICTED)
    respondWithActive([])
    const { queryClient } = renderNavigation()

    await screen.findByText('Fahrzeuge')
    await waitFor(() => expect(queryClient.getQueryState(['modules'])?.status).toBe('success'))
    expect(screen.queryByText('Karte')).not.toBeInTheDocument()
    expect(screen.getByText('Sensoren')).toBeInTheDocument()
    expect(screen.queryByText('Bäume')).not.toBeInTheDocument()
    expect(screen.queryByText('Bewässerungsgruppen')).not.toBeInTheDocument()
    expect(screen.queryByText('Auswertung')).not.toBeInTheDocument()
    expect(screen.queryByText('Einsatzpläne')).not.toBeInTheDocument()
  })

  it('renders the core sections without module links while the modules request is pending', async () => {
    permissions.mockReturnValue(UNRESTRICTED)
    server.use(http.get('*/v1/modules', () => new Promise<never>(() => undefined)))
    renderNavigation()

    expect(await screen.findByText('Fahrzeuge')).toBeInTheDocument()
    expect(screen.queryByText('Karte')).not.toBeInTheDocument()
    expect(screen.queryByText('Bäume')).not.toBeInTheDocument()
    expect(screen.queryByText('Einsatzpläne')).not.toBeInTheDocument()
  })

  it('renders the core sections when the modules request fails', async () => {
    permissions.mockReturnValue(UNRESTRICTED)
    server.use(http.get('*/v1/modules', () => HttpResponse.json({ error: 'x' }, { status: 500 })))
    const { queryClient } = renderNavigation()

    expect(await screen.findByText('Fahrzeuge')).toBeInTheDocument()
    await waitFor(() => expect(queryClient.getQueryState(['modules'])?.status).toBe('error'))
    expect(screen.getByText('Sensoren')).toBeInTheDocument()
    expect(screen.queryByText('Bäume')).not.toBeInTheDocument()
  })

  it('keeps the module links in their old order inside the sections', async () => {
    permissions.mockReturnValue(UNRESTRICTED)
    renderNavigation()

    await screen.findByText('Bäume')
    const labels = screen.getAllByRole('link').map((link) => link.textContent)
    const position = (label: string) => labels.indexOf(label)
    expect(position('Karte')).toBeLessThan(position('Bewässerungsgruppen'))
    expect(position('Bewässerungsgruppen')).toBeLessThan(position('Bäume'))
    expect(position('Bäume')).toBeLessThan(position('Einsatzpläne'))
    expect(position('Sensoren')).toBeLessThan(position('Auswertung'))
  })

  describe('assets section', () => {
    const respondWithModules = (active: string[]) =>
      server.use(
        http.get('*/v1/modules', () =>
          HttpResponse.json({
            modules: ['irrigation', 'waste'],
            organizations: [{ organizationId: 'a', active }],
          }),
        ),
      )

    it('lists the waste bins under the assets headline while the module is active', async () => {
      permissions.mockReturnValue(new Set(['waste_bin:read']))
      respondWithModules(['waste'])
      renderNavigation()

      expect(await screen.findByText('Bestand')).toBeInTheDocument()
      expect(screen.getByText('Abfallbehälter')).toBeInTheDocument()
    })

    it('omits the section while the waste module is not active', async () => {
      permissions.mockReturnValue(UNRESTRICTED)
      respondWithModules(['irrigation'])
      renderNavigation()

      await screen.findByText('Bäume')
      expect(screen.queryByText('Bestand')).not.toBeInTheDocument()
    })

    it('omits the section without the right to read waste bins', async () => {
      permissions.mockReturnValue(new Set(['tree:read']))
      respondWithModules(['irrigation', 'waste'])
      renderNavigation()

      await screen.findByText('Bäume')
      expect(screen.queryByText('Bestand')).not.toBeInTheDocument()
      expect(screen.queryByText('Abfallbehälter')).not.toBeInTheDocument()
    })
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
    const { queryClient } = renderNavigation()

    await screen.findByText('Sensoren')
    await settledPluginViews(queryClient)
    expect(screen.queryByText('Plugins')).not.toBeInTheDocument()
  })

  it('shows no plugin section when the plugin feature is off', async () => {
    permissions.mockReturnValue(UNRESTRICTED)
    server.use(
      http.get('/api-local/v1/plugins/views', () =>
        HttpResponse.json({ error: 'disabled', code: 'feature.plugins_disabled' }, { status: 503 }),
      ),
    )
    const { queryClient } = renderNavigation()

    await screen.findByText('Sensoren')
    await settledPluginViews(queryClient)
    expect(screen.queryByText('Plugins')).not.toBeInTheDocument()
  })

  it('shows the instance entries and no operational ones in admin mode', async () => {
    permissions.mockReturnValue(UNRESTRICTED)
    adminMode.value = true
    respondWithServices(true)

    renderNavigation()

    expect(await screen.findByText('Mandanten')).toBeInTheDocument()
    expect(screen.getByText('Instanz')).toBeInTheDocument()
    expect(screen.getByText('Organisation')).toBeInTheDocument()
    expect(screen.getByText('Team & Rollen')).toBeInTheDocument()
    expect(await screen.findByText('Plugins')).toBeInTheDocument()
    expect(screen.getByText('Systeminformationen')).toBeInTheDocument()
    expect(screen.getByText('Handbuch')).toBeInTheDocument()
    expect(screen.queryByText('Karte')).not.toBeInTheDocument()
    expect(screen.queryByText('Fahrzeuge')).not.toBeInTheDocument()
    expect(screen.queryByText('Sensoren')).not.toBeInTheDocument()
    expect(screen.queryByText('Einsatzpläne')).not.toBeInTheDocument()
  })

  it('filters the admin entries by permission and feature', async () => {
    permissions.mockReturnValue(new Set(['organization:read']))
    adminMode.value = true
    respondWithServices(false)

    renderNavigation()

    expect(await screen.findByText('Organisation')).toBeInTheDocument()
    expect(screen.getByText('Mandanten')).toBeInTheDocument()
    expect(screen.queryByText('Team & Rollen')).not.toBeInTheDocument()
    expect(screen.queryByText('Plugins')).not.toBeInTheDocument()
  })
})
