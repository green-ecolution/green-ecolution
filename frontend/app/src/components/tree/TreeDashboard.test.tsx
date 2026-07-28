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
import { UNRESTRICTED, type Permissions } from '@/lib/auth/permissions'

const permissions = vi.fn((): Permissions => new Set<string>())

vi.mock('@/lib/auth/usePermissions', () => ({
  usePermissions: () => permissions(),
}))

const { default: TreeDashboard } = await import('./TreeDashboard')

const tree = {
  id: 1,
  number: '42',
  species: 'Eiche',
  latitude: 54.7,
  longitude: 9.4,
  provider: null,
  description: '',
  treeClusterId: null,
  sensor: null,
} as unknown as import('@/api/backendApi').Tree

const renderDashboard = () => {
  const rootRoute = createRootRoute({ component: () => <Outlet /> })
  const indexRoute = createRoute({
    getParentRoute: () => rootRoute,
    path: '/',
    component: () => <TreeDashboard tree={tree} />,
  })
  const router = createRouter({
    routeTree: rootRoute.addChildren([indexRoute]),
    history: createMemoryHistory({ initialEntries: ['/'] }),
  })
  return render(<RouterProvider router={router} />)
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
