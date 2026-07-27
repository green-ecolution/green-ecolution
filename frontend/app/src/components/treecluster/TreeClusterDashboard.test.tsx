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

const renderDashboard = () => {
  const rootRoute = createRootRoute({ component: () => <Outlet /> })
  const indexRoute = createRoute({
    getParentRoute: () => rootRoute,
    path: '/',
    component: () => <TreeClusterDashboard treecluster={treecluster} />,
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
