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
import { type Permissions } from '@/lib/auth/permissions'

const permissions = vi.fn((): Permissions => new Set<string>())

vi.mock('@/lib/auth/usePermissions', () => ({
  usePermissions: () => permissions(),
}))

// MapLibre needs a WebGL context, which jsdom does not provide.
vi.mock('@/components/map-gl/MapPreview', () => ({
  default: () => <div data-testid="map-preview" />,
}))

const { default: TreeClusterDashboard } = await import('./TreeClusterDashboard')

const treecluster = {
  id: 1,
  name: 'Gruppe A',
  address: 'Musterweg 1',
  description: '',
  wateringStatus: 'good',
  region: { id: 1, name: 'Nord' },
  trees: [],
} as unknown as import('@/api/backendApi').TreeCluster

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
    }) as unknown as import('@/api/backendApi').TreeCluster

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
