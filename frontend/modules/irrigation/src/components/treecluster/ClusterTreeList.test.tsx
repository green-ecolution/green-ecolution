/* eslint-disable @typescript-eslint/ban-ts-comment */
// @ts-nocheck - Tests use ad-hoc routes not in the generated route tree
import { describe, it, expect, afterEach } from 'vitest'
import { render, screen, waitFor, cleanup } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
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
import type { Tree } from '../../api/client'

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
    expect(screen.getByText(/ID B-001/)).toBeInTheDocument()
    expect(screen.getByText('Eiche')).toBeInTheDocument()
    expect(screen.getByText(/ID B-002/)).toBeInTheDocument()
  })

  it('marks sensor trees with a badge and the others with "kein Sensor"', async () => {
    renderWithRouter([
      makeTree({ id: '1', species: 'Ahorn', number: 'B-001', sensor: { id: 'sensor-1' } }),
      makeTree({ id: '2', species: 'Eiche', number: 'B-002', sensor: null }),
    ])

    await waitFor(() => {
      expect(screen.getByText('Ahorn')).toBeInTheDocument()
    })

    expect(screen.getAllByText('Sensor-Baum')).toHaveLength(1)
    expect(screen.getAllByText('kein Sensor')).toHaveLength(1)
  })

  it('lists sensor trees first', async () => {
    renderWithRouter([
      makeTree({ id: '1', species: 'Ahorn' }),
      makeTree({ id: '2', species: 'Eiche', sensor: { id: 'sensor-1' } }),
    ])

    await waitFor(() => {
      expect(screen.getByText('Eiche')).toBeInTheDocument()
    })

    const links = screen.getAllByRole('link')
    expect(links[0]).toHaveAttribute('href', '/trees/2')
    expect(links[1]).toHaveAttribute('href', '/trees/1')
  })

  it('collapses to three trees and expands inline', async () => {
    const user = userEvent.setup()
    renderWithRouter(
      Array.from({ length: 5 }, (_, i) =>
        makeTree({ id: String(i + 1), species: `Baum ${i + 1}` }),
      ),
    )

    await waitFor(() => {
      expect(screen.getByText('+ 2 weitere Bäume')).toBeInTheDocument()
    })
    expect(screen.getAllByRole('link')).toHaveLength(3)

    await user.click(screen.getByRole('button', { name: 'alle anzeigen' }))
    expect(screen.getAllByRole('link')).toHaveLength(5)
    expect(screen.queryByText('+ 2 weitere Bäume')).not.toBeInTheDocument()

    await user.click(screen.getByRole('button', { name: 'weniger anzeigen' }))
    expect(screen.getAllByRole('link')).toHaveLength(3)
  })

  it('shows no toggle for three or fewer trees', async () => {
    renderWithRouter(
      Array.from({ length: 3 }, (_, i) =>
        makeTree({ id: String(i + 1), species: `Baum ${i + 1}` }),
      ),
    )

    await waitFor(() => {
      expect(screen.getAllByRole('link')).toHaveLength(3)
    })
    expect(screen.queryByRole('button')).not.toBeInTheDocument()
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
