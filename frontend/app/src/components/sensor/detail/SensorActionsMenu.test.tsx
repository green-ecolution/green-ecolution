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

vi.mock('./SensorActionsContext', () => ({
  useSensorActions: () => ({
    requestActivate: vi.fn(),
    requestReassign: vi.fn(),
    requestRemove: vi.fn(),
  }),
}))

const { default: SensorActionsMenu } = await import('./SensorActionsMenu')

const sensor = { id: 'eui-1', status: 'active' } as unknown as import('@/api/backendApi').Sensor

const renderMenu = async () => {
  const queryClient = new QueryClient({ defaultOptions: { queries: { retry: false } } })
  const rootRoute = createRootRoute({ component: () => <Outlet /> })
  const indexRoute = createRoute({
    getParentRoute: () => rootRoute,
    path: '/',
    component: () => (
      <QueryClientProvider client={queryClient}>
        <SensorActionsMenu sensor={sensor} />
      </QueryClientProvider>
    ),
  })
  const router = createRouter({
    routeTree: rootRoute.addChildren([indexRoute]),
    history: createMemoryHistory({ initialEntries: ['/'] }),
  })
  // preload route matches so the initial render below isn't left pending
  await router.load()
  return render(<RouterProvider router={router} />)
}

describe('SensorActionsMenu', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    permissions.mockReturnValue(new Set<string>())
  })

  it('hides the trigger without sensor:update or sensor:delete', async () => {
    permissions.mockReturnValue(new Set(['sensor:read']))
    await renderMenu()
    expect(screen.queryByRole('button', { name: 'Aktionen' })).not.toBeInTheDocument()
  })

  it('shows the trigger with sensor:update', async () => {
    permissions.mockReturnValue(new Set(['sensor:update']))
    await renderMenu()
    expect(await screen.findByRole('button', { name: 'Aktionen' })).toBeInTheDocument()
  })

  it('shows the trigger with only sensor:delete', async () => {
    permissions.mockReturnValue(new Set(['sensor:delete']))
    await renderMenu()
    expect(await screen.findByRole('button', { name: 'Aktionen' })).toBeInTheDocument()
  })
})
