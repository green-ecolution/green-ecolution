/* eslint-disable @typescript-eslint/ban-ts-comment */
// @ts-nocheck - ad-hoc routes are not part of the generated route tree
import { describe, it, expect } from 'vitest'
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

  return render(<RouterProvider router={router} />)
}

describe('Forbidden', () => {
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
})
