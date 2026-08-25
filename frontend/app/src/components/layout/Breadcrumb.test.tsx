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

import Breadcrumb from './Breadcrumb'

const renderBreadcrumb = () => {
  const rootRoute = createRootRoute({ component: () => <Outlet /> })
  const treesRoute = createRoute({
    getParentRoute: () => rootRoute,
    path: 'trees',
    loader: () => ({ crumb: { title: 'Bäume' } }),
    component: () => <Outlet />,
  })
  const treeDetailRoute = createRoute({
    getParentRoute: () => treesRoute,
    path: '$treeId',
    loader: () => ({ crumb: { title: 'Eiche #1234' } }),
    component: () => <Breadcrumb />,
  })
  const router = createRouter({
    routeTree: rootRoute.addChildren([treesRoute.addChildren([treeDetailRoute])]),
    history: createMemoryHistory({ initialEntries: ['/trees/1234'] }),
  })

  return render(<RouterProvider router={router} />)
}

describe('Breadcrumb', () => {
  it('renders the whole trail', async () => {
    renderBreadcrumb()

    await waitFor(() => {
      expect(screen.getByText('Dashboard')).toBeInTheDocument()
    })
    expect(screen.getByText('Bäume')).toBeInTheDocument()
    expect(screen.getByText('Eiche #1234')).toBeInTheDocument()
  })

  it('keeps separators as siblings so the list stays valid HTML', async () => {
    const { container } = renderBreadcrumb()

    await waitFor(() => {
      expect(screen.getByText('Eiche #1234')).toBeInTheDocument()
    })

    expect(container.querySelectorAll('li li')).toHaveLength(0)

    const list = container.querySelector('ol')
    const separators = list.querySelectorAll(':scope > li[role="presentation"]')
    expect(separators).toHaveLength(2)
  })
})
