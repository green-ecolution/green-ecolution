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
import ClusterTreeList from './ClusterTreeList'
import { WateringStatus } from '@/api/backendApi'
import type { Tree } from '@/api/backendApi'

function makeTree(overrides: Partial<Tree> = {}): Tree {
  return {
    id: '1',
    species: 'Ahorn',
    number: 'B-001',
    wateringStatus: WateringStatus.Good,
    sensor: null,
    ...overrides,
  } as Tree
}

function renderWithRouter(trees: Tree[]) {
  const rootRoute = createRootRoute({ component: () => <Outlet /> })

  const indexRoute = createRoute({
    getParentRoute: () => rootRoute,
    path: '/',
    component: () => <ClusterTreeList trees={trees} />,
  })

  const detailRoute = createRoute({
    getParentRoute: () => rootRoute,
    path: '/trees/$treeId',
    component: () => <div data-testid="detail-page">Detail</div>,
  })

  const routeTree = rootRoute.addChildren([indexRoute, detailRoute])
  const router = createRouter({
    routeTree,
    history: createMemoryHistory({ initialEntries: ['/'] }),
  })

  return render(<RouterProvider router={router} />)
}

describe('ClusterTreeList', () => {
  afterEach(cleanup)

  it('renders one row per tree with species and number', async () => {
    renderWithRouter([
      makeTree({ id: '1', species: 'Ahorn', number: 'B-001' }),
      makeTree({ id: '2', species: 'Eiche', number: 'B-002' }),
    ])

    await waitFor(() => {
      expect(screen.getByText('Bäume · 2')).toBeInTheDocument()
    })

    expect(screen.getByText('Ahorn')).toBeInTheDocument()
    expect(screen.getByText('B-001')).toBeInTheDocument()
    expect(screen.getByText('Eiche')).toBeInTheDocument()
    expect(screen.getByText('B-002')).toBeInTheDocument()
  })

  it('shows a Sensor badge only for trees with a sensor', async () => {
    renderWithRouter([
      makeTree({ id: '1', species: 'Ahorn', number: 'B-001', sensor: { id: 'sensor-1' } }),
      makeTree({ id: '2', species: 'Eiche', number: 'B-002', sensor: null }),
    ])

    await waitFor(() => {
      expect(screen.getByText('Ahorn')).toBeInTheDocument()
    })

    expect(screen.getAllByText('Sensor')).toHaveLength(1)
  })

  it('links each row to the tree detail route', async () => {
    renderWithRouter([makeTree({ id: '42', species: 'Ahorn', number: 'B-001' })])

    await waitFor(() => {
      expect(screen.getByText('Ahorn')).toBeInTheDocument()
    })

    const link = screen.getByRole('link')
    expect(link).toHaveAttribute('href', '/trees/42')
  })

  it('renders the empty state when there are no trees', async () => {
    renderWithRouter([])

    await waitFor(() => {
      expect(
        screen.getByText('Der Bewässerungsgruppe wurden keine Bäume hinzugefügt.'),
      ).toBeInTheDocument()
    })
  })
})
