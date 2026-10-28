/* eslint-disable @typescript-eslint/ban-ts-comment */
// @ts-nocheck - ad-hoc routes are not part of the generated route tree
import { describe, it, expect, beforeEach } from 'vitest'
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
import { userQueries } from '@/api/queries'
import { getOrganizationScope, setOrganizationScope } from '@/lib/auth/scope/organizationScopeStore'
import Forbidden from './Forbidden'

const renderForbidden = () => {
  const rootRoute = createRootRoute({ component: () => <Outlet /> })

  const indexRoute = createRoute({
    getParentRoute: () => rootRoute,
    path: '/',
    component: () => <Forbidden />,
  })

  const dashboardRoute = createRoute({
    getParentRoute: () => rootRoute,
    path: '/dashboard',
    component: () => <div data-testid="dashboard-page">Dashboard</div>,
  })

  const router = createRouter({
    routeTree: rootRoute.addChildren([indexRoute, dashboardRoute]),
    history: createMemoryHistory({ initialEntries: ['/'] }),
  })

  const queryClient = new QueryClient({ defaultOptions: { queries: { staleTime: Infinity } } })
  queryClient.setQueryData(userQueries.me().queryKey, { id: 'u1' })
  return render(
    <QueryClientProvider client={queryClient}>
      <RouterProvider router={router} />
    </QueryClientProvider>,
  )
}

describe('Forbidden', () => {
  beforeEach(() => {
    localStorage.clear()
    setOrganizationScope('u1', null)
  })

  it('states that access is denied', async () => {
    renderForbidden()

    await waitFor(() => {
      expect(screen.getByRole('heading', { name: 'Kein Zugriff' })).toBeInTheDocument()
    })
    expect(screen.getByText(/fehlt dir die nötige Berechtigung/)).toBeInTheDocument()
  })

  it('offers a way back to the dashboard', async () => {
    const user = userEvent.setup()
    renderForbidden()

    await waitFor(() => {
      expect(screen.getByText('Zum Dashboard')).toBeInTheDocument()
    })

    await user.click(screen.getByText('Zum Dashboard'))

    await waitFor(() => {
      expect(screen.getByTestId('dashboard-page')).toBeInTheDocument()
    })
  })

  it('offers no scope switch without a selected organization', async () => {
    renderForbidden()

    await screen.findByText('Zum Dashboard')
    expect(
      screen.queryByRole('button', { name: 'Zu meiner Organisation wechseln' }),
    ).not.toBeInTheDocument()
  })

  it('switches back to the own organization when a scope hides the page', async () => {
    const user = userEvent.setup()
    setOrganizationScope('u1', 'firma_a')
    renderForbidden()

    await user.click(await screen.findByRole('button', { name: 'Zu meiner Organisation wechseln' }))

    await waitFor(() => expect(getOrganizationScope()).toBeNull())
  })
})
