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
import ClusterSensorCard from './ClusterSensorCard'
import type { Tree } from '../../api/client'

function makeTree(overrides: Partial<Tree> = {}): Tree {
  return {
    id: '1',
    species: 'Quercus robur',
    number: '1006',
    sensor: null,
    ...overrides,
  } as Tree
}

const sensor = (id: string, overrides = {}) => ({
  id,
  latestData: null,
  dataHealth: 'ok',
  implausibleRecent: 0,
  ...overrides,
})

function renderWithRouter(trees: Tree[]) {
  const rootRoute = createRootRoute({ component: () => <Outlet /> })
  const indexRoute = createRoute({
    getParentRoute: () => rootRoute,
    path: '/',
    component: () => <ClusterSensorCard trees={trees} />,
  })
  const sensorRoute = createRoute({
    getParentRoute: () => rootRoute,
    path: '/sensors/$sensorId',
    component: () => <div>Sensor</div>,
  })
  const router = createRouter({
    routeTree: rootRoute.addChildren([indexRoute, sensorRoute]),
    history: createMemoryHistory({ initialEntries: ['/'] }),
  })
  return render(<RouterProvider router={router} />)
}

describe('ClusterSensorCard', () => {
  afterEach(cleanup)

  it('renders one compact row per sensor tree linking to the sensor', async () => {
    renderWithRouter([
      makeTree({ id: '1', number: '1006', sensor: sensor('s-1') }),
      makeTree({ id: '2', number: '1005' }),
    ])

    await waitFor(() => {
      expect(screen.getByText('Quercus robur')).toBeInTheDocument()
    })

    expect(screen.getByText(/ID 1006/)).toBeInTheDocument()
    expect(screen.queryByText(/ID 1005/)).not.toBeInTheDocument()
    expect(screen.getByRole('link')).toHaveAttribute('href', '/sensors/s-1')
    expect(screen.getByText('Keine Daten')).toBeInTheDocument()
    expect(screen.getByText('1 von 2 Bäumen mit Sensor')).toBeInTheDocument()
  })

  it('shows a data quality badge only for sensors with a warning', async () => {
    renderWithRouter([
      makeTree({ id: '1', number: '1006', sensor: sensor('s-1', { dataHealth: 'suspect' }) }),
      makeTree({ id: '2', number: '1001', sensor: sensor('s-2') }),
    ])

    await waitFor(() => {
      expect(screen.getByText('Datenqualität prüfen')).toBeInTheDocument()
    })
    expect(screen.queryByText('Daten plausibel')).not.toBeInTheDocument()
  })

  it('renders the notice when no tree has a sensor', async () => {
    renderWithRouter([makeTree()])

    await waitFor(() => {
      expect(
        screen.getByText('Kein Baum dieser Gruppe ist mit einem Sensor ausgestattet.'),
      ).toBeInTheDocument()
    })
  })
})
