/* eslint-disable @typescript-eslint/ban-ts-comment */
// @ts-nocheck - Tests use ad-hoc routes not in the generated route tree
import { describe, it, expect, afterEach } from 'vitest'
import { render, screen, waitFor, cleanup } from '@testing-library/react'
import {
  createRootRoute,
  createRoute,
  createRouter,
  createMemoryHistory,
  RouterProvider,
  Outlet,
} from '@tanstack/react-router'
import ClusterCard from './ClusterCard'
import { SoilCondition, WateringStatus } from '@/api/backendApi'
import type { TreeClusterInList } from '@/api/backendApi'

function makeCluster(overrides: Partial<TreeClusterInList> = {}): TreeClusterInList {
  return {
    id: '42',
    name: 'Gruppe Hafermarkt',
    address: 'Hafermarkt 5, Flensburg',
    description: '',
    archived: false,
    createdAt: '2024-01-01T00:00:00Z',
    updatedAt: '2024-01-01T00:00:00Z',
    latitude: 54.78,
    longitude: 9.43,
    moistureLevel: 0,
    wateringStatus: WateringStatus.Bad,
    region: { id: '1', name: 'Mürwik' },
    treeIds: ['1', '2', '3'],
    sensorCount: 1,
    soilCondition: SoilCondition.Ss,
    lastWatered: null,
    ...overrides,
  }
}

function renderWithRouter(cluster: TreeClusterInList) {
  const rootRoute = createRootRoute({ component: () => <Outlet /> })

  const indexRoute = createRoute({
    getParentRoute: () => rootRoute,
    path: '/',
    component: () => <ClusterCard treecluster={cluster} />,
  })

  const detailRoute = createRoute({
    getParentRoute: () => rootRoute,
    path: '/treecluster/$treeclusterId',
    component: () => <div data-testid="detail-page">Detail</div>,
  })

  const routeTree = rootRoute.addChildren([indexRoute, detailRoute])
  const router = createRouter({
    routeTree,
    history: createMemoryHistory({ initialEntries: ['/'] }),
  })

  return render(<RouterProvider router={router} />)
}

describe('ClusterCard', () => {
  afterEach(cleanup)

  it('renders name, status label, tree count and a Details link', async () => {
    renderWithRouter(makeCluster())

    await waitFor(() => {
      expect(screen.getByText('Gruppe Hafermarkt')).toBeInTheDocument()
    })

    expect(screen.getByText('Sehr trocken')).toBeInTheDocument()
    expect(screen.getByText('3')).toBeInTheDocument()
    expect(screen.getByText('Details')).toBeInTheDocument()

    const link = screen.getByRole('link', { name: /Bewässerungsgruppe Gruppe Hafermarkt/i })
    expect(link).toHaveAttribute('href', '/treecluster/42')
  })

  it('shows the "noch nicht bewässert" footer when never watered', async () => {
    renderWithRouter(makeCluster({ lastWatered: null }))

    await waitFor(() => {
      expect(screen.getByText(/noch nicht bewässert/)).toBeInTheDocument()
    })
  })
})
