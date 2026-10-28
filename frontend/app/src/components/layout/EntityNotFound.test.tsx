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
import EntityNotFound from './EntityNotFound'

const LEAVE_SCOPE = 'Zu allen meinen Organisationen wechseln'

const renderNotFound = () => {
  const rootRoute = createRootRoute({ component: () => <Outlet /> })
  const indexRoute = createRoute({
    getParentRoute: () => rootRoute,
    path: '/',
    component: () => (
      <EntityNotFound entityName="Fahrzeug" backTo="/vehicles" backLabel="Zu den Fahrzeugen" />
    ),
  })
  const vehiclesRoute = createRoute({
    getParentRoute: () => rootRoute,
    path: '/vehicles',
    component: () => <div>Fahrzeugliste</div>,
  })
  const router = createRouter({
    routeTree: rootRoute.addChildren([indexRoute, vehiclesRoute]),
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

describe('EntityNotFound', () => {
  beforeEach(() => {
    localStorage.clear()
    setOrganizationScope('u1', null)
  })

  it('offers no scope switch without a selected organization', async () => {
    renderNotFound()

    await screen.findByText('Zu den Fahrzeugen')
    expect(screen.queryByRole('button', { name: LEAVE_SCOPE })).not.toBeInTheDocument()
  })

  it('switches to all organizations when a scope hides the object', async () => {
    const user = userEvent.setup()
    setOrganizationScope('u1', 'firma_a')
    renderNotFound()

    await user.click(await screen.findByRole('button', { name: LEAVE_SCOPE }))

    await waitFor(() => expect(getOrganizationScope()).toBeNull())
    expect(localStorage.getItem('ge.organizationScope.u1')).toBeNull()
  })
})
