/* eslint-disable @typescript-eslint/ban-ts-comment */
// @ts-nocheck - ad-hoc routes are not part of the generated route tree
import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import {
  createRootRoute,
  createRoute,
  createRouter,
  createMemoryHistory,
  RouterProvider,
  Outlet,
} from '@tanstack/react-router'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { type Permissions } from '@/lib/auth/permissions'

const permissions = vi.fn((): Permissions => new Set<string>())

vi.mock('@/lib/auth/usePermissions', () => ({
  usePermissions: () => permissions(),
}))

// The CommentsSection at the bottom of the page needs a session context and
// its own user lookup; neither is under test here.
vi.mock('@/lib/auth/authSessionContext', () => ({
  useAuthSession: () => ({ isAuthenticated: true, accessToken: null }),
}))

const getMe = vi.fn().mockResolvedValue({ id: 'me', roles: [] })
const listOrganizations = vi.fn()
vi.mock('@/api/backendApi', async (importOriginal) => {
  const actual = await importOriginal<typeof import('@/api/backendApi')>()
  return {
    ...actual,
    userApi: {
      getMe: (...args: unknown[]) => getMe(...args) as unknown,
      listMyOrganizations: () => [],
    },
    organizationApi: { listOrganizations: () => listOrganizations() as unknown },
  }
})

// MapLibre needs a WebGL context, which jsdom does not provide.
vi.mock('@/components/map-gl/MapPreview', () => ({
  default: () => <div data-testid="map-preview" />,
}))

const { default: TreeClusterDashboard } = await import('./TreeClusterDashboard')
const { clusterApi } = await import('../../api/client')

const treecluster = {
  id: 1,
  name: 'Gruppe A',
  address: 'Musterweg 1',
  description: '',
  wateringStatus: 'good',
  region: { id: 1, name: 'Nord' },
  trees: [],
} as unknown as import('../../api/client').TreeCluster

const renderDashboard = (cluster = treecluster) => {
  const rootRoute = createRootRoute({ component: () => <Outlet /> })
  const indexRoute = createRoute({
    getParentRoute: () => rootRoute,
    path: '/',
    component: () => <TreeClusterDashboard treecluster={cluster} />,
  })
  const router = createRouter({
    routeTree: rootRoute.addChildren([indexRoute]),
    history: createMemoryHistory({ initialEntries: ['/'] }),
  })
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false } },
  })
  return render(
    <QueryClientProvider client={queryClient}>
      <RouterProvider router={router} />
    </QueryClientProvider>,
  )
}

describe('TreeClusterDashboard actions', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    permissions.mockReturnValue(new Set<string>())
  })

  it('shows edit but not delete with only tree_cluster:update', async () => {
    permissions.mockReturnValue(new Set(['tree_cluster:update']))
    renderDashboard()
    expect(await screen.findByText('Gruppe bearbeiten')).toBeInTheDocument()
    expect(screen.queryByLabelText('Weitere Aktionen')).not.toBeInTheDocument()
  })

  it('shows the actions menu with tree_cluster:delete', async () => {
    permissions.mockReturnValue(new Set(['tree_cluster:delete']))
    renderDashboard()
    expect(await screen.findByLabelText('Weitere Aktionen')).toBeInTheDocument()
    expect(screen.queryByText('Gruppe bearbeiten')).not.toBeInTheDocument()
  })

  it('renders no actions block when neither is permitted', async () => {
    permissions.mockReturnValue(new Set(['tree_cluster:read']))
    renderDashboard()
    await screen.findByText(/Bewässerungsgruppe: Gruppe A/)
    expect(screen.queryByText('Gruppe bearbeiten')).not.toBeInTheDocument()
    expect(screen.queryByLabelText('Weitere Aktionen')).not.toBeInTheDocument()
  })
})

describe('TreeClusterDashboard unknown-status notice', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    permissions.mockReturnValue(new Set<string>())
  })

  const withTrees = (overrides: object, trees: object[]) =>
    ({
      ...treecluster,
      ...overrides,
      trees,
    }) as unknown as import('../../api/client').TreeCluster

  const sensorlessTree = (id: string) => ({
    id,
    species: 'Ahorn',
    number: `B-${id}`,
    wateringStatus: 'unknown',
    plantingYear: 2024,
    sensor: null,
  })

  it('explains a missing soil type', async () => {
    const cluster = withTrees({ wateringStatus: 'unknown', soilCondition: 'unknown' }, [
      {
        ...sensorlessTree('a'),
        sensor: {
          id: 'eui-a',
          status: 'online',
          model: {
            id: 'model-1',
            name: 'GES-1000',
            abilities: [{ id: 'ab-1', ability: 'soil_moisture', unit: 'percent', depthCm: 40 }],
          },
        },
      },
    ])

    renderDashboard(cluster)

    expect(
      await screen.findByText('Warum ist der Bewässerungszustand unbekannt?'),
    ).toBeInTheDocument()
    expect(
      screen.getByText(/Bodenbeschaffenheit der Gruppe ist nicht bestimmt/),
    ).toBeInTheDocument()
  })

  it('explains a group without any sensor', async () => {
    const cluster = withTrees({ wateringStatus: 'unknown' }, [
      sensorlessTree('a'),
      sensorlessTree('b'),
    ])

    renderDashboard(cluster)

    expect(
      await screen.findByText('Warum ist der Bewässerungszustand unbekannt?'),
    ).toBeInTheDocument()
    expect(
      screen.getByText('Diese Gruppe hat keinen Sensor, daher liegen keine Messwerte vor.'),
    ).toBeInTheDocument()
  })

  it('stays hidden while the status is known', async () => {
    renderDashboard(withTrees({ wateringStatus: 'good' }, [sensorlessTree('a')]))

    await screen.findByText(/Bewässerungsgruppe: Gruppe A/)
    expect(
      screen.queryByText('Warum ist der Bewässerungszustand unbekannt?'),
    ).not.toBeInTheDocument()
  })

  it('leaves the empty-cluster case to its own notice', async () => {
    renderDashboard(withTrees({ wateringStatus: 'unknown' }, []))

    expect(await screen.findByText('Keine Bäume zugewiesen')).toBeInTheDocument()
    expect(
      screen.queryByText('Warum ist der Bewässerungszustand unbekannt?'),
    ).not.toBeInTheDocument()
  })
})

describe('TreeClusterDashboard transfer', () => {
  const orgs = [
    { id: 'root', name: 'Green Ecolution', parentId: null, memberCount: 0 },
    { id: 'tbz', name: 'Betriebshof', parentId: 'root', memberCount: 0 },
  ]
  const meWithUpdateIn = (organizationId: string) => ({
    id: 'me',
    organization: orgs[0],
    roles: [{ id: 'r', name: 'Rolle', organizationId, permissions: ['tree_cluster:update'] }],
  })

  beforeEach(() => {
    vi.clearAllMocks()
    permissions.mockReturnValue(new Set(['tree_cluster:update', 'organization:read']))
    listOrganizations.mockResolvedValue(orgs)
  })

  it('moves the cluster to the chosen organization', async () => {
    const user = userEvent.setup()
    getMe.mockResolvedValue(meWithUpdateIn('root'))
    const transfer = vi.spyOn(clusterApi, 'transferCluster').mockResolvedValue(undefined)
    renderDashboard({ ...treecluster, organizationId: 'tbz' })

    await user.click(await screen.findByLabelText('Weitere Aktionen'))
    await user.click(
      await screen.findByRole('menuitem', { name: 'In andere Organisation verschieben' }),
    )
    await user.click(screen.getByRole('combobox', { name: /zielorganisation/i }))
    await user.click(screen.getByRole('option', { name: 'Green Ecolution' }))
    await user.click(screen.getByRole('button', { name: 'Verschieben' }))

    await waitFor(() =>
      expect(transfer).toHaveBeenCalledWith({
        clusterId: '1',
        transferRequest: { organizationId: 'root' },
      }),
    )
  })

  it('offers no move without another organization to move to', async () => {
    const user = userEvent.setup()
    getMe.mockResolvedValue(meWithUpdateIn('tbz'))
    permissions.mockReturnValue(
      new Set(['tree_cluster:update', 'tree_cluster:delete', 'organization:read']),
    )
    renderDashboard({ ...treecluster, organizationId: 'tbz' })

    await user.click(await screen.findByLabelText('Weitere Aktionen'))
    expect(await screen.findByRole('menuitem', { name: 'Gruppe löschen' })).toBeInTheDocument()
    expect(
      screen.queryByRole('menuitem', { name: 'In andere Organisation verschieben' }),
    ).not.toBeInTheDocument()
  })
})
