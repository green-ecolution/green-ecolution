import { describe, it, expect, vi, afterEach, beforeEach } from 'vitest'
import { render, screen, waitFor, cleanup } from '@testing-library/react'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import {
  createRootRoute,
  createRoute,
  createRouter,
  createMemoryHistory,
  RouterProvider,
  Outlet,
} from '@tanstack/react-router'

const getClusterSoilMoisture = vi.fn()
vi.mock('@/api/backendApi', async (importOriginal) => {
  const actual = await importOriginal<typeof import('@/api/backendApi')>()
  return {
    ...actual,
    clusterApi: {
      ...actual.clusterApi,
      getClusterSoilMoisture: (...args: unknown[]) => getClusterSoilMoisture(...args) as unknown,
    },
  }
})

// Mocking clusterApi rather than pre-seeding the QueryClient cache: it exercises
// the real clusterSoilMoistureQuery/useQuery wiring instead of relying on an
// exact hand-built query key that could silently drift from the real one.
import ClusterWateringHistory from './ClusterWateringHistory'

const CLUSTER_ID = '11111111-1111-4111-8111-111111111111'

function renderWithProviders() {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false, gcTime: 0, staleTime: 0 } },
  })

  const rootRoute = createRootRoute({ component: () => <Outlet /> })
  const indexRoute = createRoute({
    getParentRoute: () => rootRoute,
    path: '/',
    component: () => <ClusterWateringHistory clusterId={CLUSTER_ID} />,
  })
  const detailRoute = createRoute({
    getParentRoute: () => rootRoute,
    path: '/watering-plans/$wateringPlanId',
    component: () => <div data-testid="detail-page">Detail</div>,
  })
  const routeTree = rootRoute.addChildren([indexRoute, detailRoute])
  const router = createRouter({
    routeTree,
    history: createMemoryHistory({ initialEntries: ['/'] }),
  })

  return render(
    <QueryClientProvider client={queryClient}>
      <RouterProvider router={router} />
    </QueryClientProvider>,
  )
}

beforeEach(() => vi.clearAllMocks())
afterEach(cleanup)

describe('ClusterWateringHistory', () => {
  it('renders one row per watering event with date, link and formatted liters', async () => {
    getClusterSoilMoisture.mockResolvedValue({
      bucket: 'day',
      series: [],
      thresholds: [],
      wateringEvents: [
        {
          wateringPlanId: 'plan-1',
          date: new Date('2024-03-05T00:00:00Z'),
          consumedWaterLiters: 1234,
        },
        {
          wateringPlanId: 'plan-2',
          date: new Date('2024-02-20T00:00:00Z'),
          consumedWaterLiters: 500,
        },
      ],
    })

    renderWithProviders()

    await waitFor(() => {
      expect(screen.getByText('05.03.2024')).toBeInTheDocument()
    })

    expect(screen.getByText('20.02.2024')).toBeInTheDocument()
    expect(screen.getByText('1.234 L')).toBeInTheDocument()
    expect(screen.getByText('500 L')).toBeInTheDocument()

    const links = screen.getAllByRole('link', { name: 'Einsatz ansehen' })
    expect(links).toHaveLength(2)
    expect(links[0]).toHaveAttribute('href', '/watering-plans/plan-1')
    expect(links[1]).toHaveAttribute('href', '/watering-plans/plan-2')
  })

  it('caps the list at the 6 newest events', async () => {
    // Newest-first, matching the real API's ORDER BY wp.date DESC contract —
    // asserting both presence of the newest and absence of the oldest is
    // what actually distinguishes slice(0, 6) from slice(-6).
    const wateringEvents = Array.from({ length: 8 }, (_, i) => ({
      wateringPlanId: `plan-${i}`,
      date: new Date(2024, 0, 8 - i),
      consumedWaterLiters: 100 + i,
    }))
    getClusterSoilMoisture.mockResolvedValue({
      bucket: 'day',
      series: [],
      thresholds: [],
      wateringEvents,
    })

    renderWithProviders()

    await waitFor(() => {
      expect(screen.getAllByRole('link', { name: 'Einsatz ansehen' })).toHaveLength(6)
    })

    const newestSixDates = [
      '08.01.2024',
      '07.01.2024',
      '06.01.2024',
      '05.01.2024',
      '04.01.2024',
      '03.01.2024',
    ]
    const oldestTwoDates = ['02.01.2024', '01.01.2024']

    for (const date of newestSixDates) {
      expect(screen.getByText(date)).toBeInTheDocument()
    }
    for (const date of oldestTwoDates) {
      expect(screen.queryByText(date)).not.toBeInTheDocument()
    }
  })

  it('shows the empty state when there are no watering events', async () => {
    getClusterSoilMoisture.mockResolvedValue({
      bucket: 'day',
      series: [],
      thresholds: [],
      wateringEvents: [],
    })

    renderWithProviders()

    await waitFor(() => {
      expect(
        screen.getByText('Für diese Gruppe wurden noch keine Einsätze abgeschlossen.'),
      ).toBeInTheDocument()
    })
  })
})
