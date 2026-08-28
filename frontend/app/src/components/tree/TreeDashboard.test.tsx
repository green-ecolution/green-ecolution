/* eslint-disable @typescript-eslint/ban-ts-comment */
// @ts-nocheck - ad-hoc routes are not part of the generated route tree
import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen } from '@testing-library/react'
import {
  createRootRoute,
  createRoute,
  createRouter,
  createMemoryHistory,
  RouterProvider,
  Outlet,
} from '@tanstack/react-router'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { UNRESTRICTED, type Permissions } from '@/lib/auth/permissions'

const permissions = vi.fn((): Permissions => new Set<string>())

vi.mock('@/lib/auth/usePermissions', () => ({
  usePermissions: () => permissions(),
}))

// MapLibre needs a WebGL context, which jsdom does not provide.
vi.mock('@/components/map-gl/MapPreview', () => ({
  default: () => <div data-testid="map-preview" />,
}))

const { default: TreeDashboard } = await import('./TreeDashboard')

const tree = {
  id: '11111111-1111-1111-1111-111111111111',
  number: '42',
  species: 'Eiche',
  plantingYear: 2019,
  latitude: 54.7,
  longitude: 9.4,
  provider: null,
  description: '',
  treeClusterId: null,
  sensor: null,
  wateringStatus: 'bad',
  updatedAt: '2026-01-02T10:00:00Z',
} as unknown as import('@/api/backendApi').Tree

const cluster = {
  id: '22222222-2222-2222-2222-222222222222',
  name: 'Gruppe A',
  address: 'Musterweg 1',
  wateringStatus: 'good',
  region: { id: '3', name: 'Nord' },
  trees: [tree],
} as unknown as import('@/api/backendApi').TreeCluster

const renderDashboard = (props = { tree }) => {
  const rootRoute = createRootRoute({ component: () => <Outlet /> })
  const indexRoute = createRoute({
    getParentRoute: () => rootRoute,
    path: '/',
    component: () => <TreeDashboard {...props} />,
  })
  const router = createRouter({
    routeTree: rootRoute.addChildren([indexRoute]),
    history: createMemoryHistory({ initialEntries: ['/'] }),
  })
  const queryClient = new QueryClient({ defaultOptions: { queries: { retry: false } } })
  return render(
    <QueryClientProvider client={queryClient}>
      <RouterProvider router={router} />
    </QueryClientProvider>,
  )
}

describe('TreeDashboard edit link', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    permissions.mockReturnValue(new Set<string>())
  })

  it('hides the edit link without tree:update', async () => {
    permissions.mockReturnValue(new Set(['tree:read']))
    renderDashboard()
    await screen.findByText('Baum: 42')
    expect(screen.queryByText('Baum bearbeiten')).not.toBeInTheDocument()
  })

  it('shows the edit link with tree:update', async () => {
    permissions.mockReturnValue(new Set(['tree:update']))
    renderDashboard()
    expect(await screen.findByText('Baum bearbeiten')).toBeInTheDocument()
  })

  it('shows the edit link for unrestricted access', async () => {
    permissions.mockReturnValue(UNRESTRICTED)
    renderDashboard()
    expect(await screen.findByText('Baum bearbeiten')).toBeInTheDocument()
  })
})

describe('TreeDashboard content', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    permissions.mockReturnValue(UNRESTRICTED)
  })

  it('explains the unknown status when no sensor is linked', async () => {
    renderDashboard()
    await screen.findByText('Baum: 42')
    expect(screen.getByText(/Dieser Baum ist mit keinem Sensor ausgestattet/)).toBeInTheDocument()
  })

  it('links the sensor when one is attached', async () => {
    const withSensor = {
      ...tree,
      sensor: { id: 'EUI-1', status: 'online', model: { id: 'model-1' }, latestData: null },
    } as unknown as import('@/api/backendApi').Tree
    renderDashboard({ tree: withSensor })
    expect(await screen.findByText('EUI-1')).toBeInTheDocument()
    expect(screen.getByText('Online')).toBeInTheDocument()
  })

  it('states that the tree has no cluster', async () => {
    renderDashboard()
    expect(await screen.findByText(/Keiner Bewässerungsgruppe zugeordnet/)).toBeInTheDocument()
    expect(screen.getByText(/Ohne Gruppe fehlt die Bodenart/)).toBeInTheDocument()
  })

  it('explains a tree status that deviates from its cluster', async () => {
    renderDashboard({ tree, treeCluster: cluster })
    expect(await screen.findByText('Gruppe A')).toBeInTheDocument()
    expect(screen.getByText(/Dieser Baum ist als »Sehr trocken« bewertet/)).toBeInTheDocument()
  })
})
