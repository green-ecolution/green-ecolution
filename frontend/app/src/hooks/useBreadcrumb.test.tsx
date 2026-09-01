/* eslint-disable @typescript-eslint/ban-ts-comment */
// @ts-nocheck - ad-hoc routes are not part of the generated route tree
import { describe, it, expect } from 'vitest'
import { render, screen, waitFor } from '@testing-library/react'
import {
  createRootRoute,
  createRoute,
  createRouter,
  createMemoryHistory,
  RouterProvider,
  Outlet,
} from '@tanstack/react-router'
import { useBreadcrumbs } from './useBreadcrumb'

function BreadcrumbList() {
  const breadcrumbs = useBreadcrumbs()
  return (
    <ul>
      {breadcrumbs.map((crumb) => (
        <li key={crumb.path}>{crumb.title}</li>
      ))}
    </ul>
  )
}

const renderBreadcrumbs = () => {
  const rootRoute = createRootRoute({ component: () => <Outlet /> })
  const treesRoute = createRoute({
    getParentRoute: () => rootRoute,
    path: 'trees',
    loader: () => ({ crumb: { titleKey: 'trees' } }),
    component: () => <Outlet />,
  })
  const treeDetailRoute = createRoute({
    getParentRoute: () => treesRoute,
    path: '$treeId',
    loader: () => ({ crumb: { title: 'Eiche #1234' } }),
    component: () => <BreadcrumbList />,
  })
  const router = createRouter({
    routeTree: rootRoute.addChildren([treesRoute.addChildren([treeDetailRoute])]),
    history: createMemoryHistory({ initialEntries: ['/trees/1234'] }),
  })

  return render(<RouterProvider router={router} />)
}

describe('useBreadcrumbs', () => {
  it('resolves a titleKey crumb through the navigation catalog', async () => {
    renderBreadcrumbs()

    await waitFor(() => {
      expect(screen.getByText('Bäume')).toBeInTheDocument()
    })
  })

  it('passes a literal title crumb through unchanged', async () => {
    renderBreadcrumbs()

    await waitFor(() => {
      expect(screen.getByText('Eiche #1234')).toBeInTheDocument()
    })
  })
})
