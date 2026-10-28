/* eslint-disable @typescript-eslint/ban-ts-comment */
// @ts-nocheck - ad-hoc routes are not part of the generated route tree
import { describe, it, expect, vi, beforeEach } from 'vitest'
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
import { type Permissions } from '@/lib/auth/permissions'

const permissions = vi.fn((): Permissions => new Set<string>())
vi.mock('@/lib/auth/usePermissions', () => ({
  usePermissions: () => permissions(),
}))

const getMe = vi.fn()
const listOrganizations = vi.fn()
vi.mock('@/api/backendApi', async (importOriginal) => ({
  ...(await importOriginal<typeof import('@/api/backendApi')>()),
  userApi: { getMe: () => getMe() as unknown, listMyOrganizations: () => [] },
  organizationApi: { listOrganizations: () => listOrganizations() as unknown },
}))

vi.mock('@/lib/auth/authSessionContext', () => ({
  useAuthSession: () => ({ isAuthenticated: true, accessToken: null }),
}))

const { default: VehicleDashboard } = await import('./VehicleDashboard')
const { vehicleApi } = await import('@/api/backendApi')

const vehicle = {
  id: 'v-1',
  numberPlate: 'FL-GE 123',
  status: 'available',
  type: 'transporter',
  model: 'Unimog',
  drivingLicense: 'B',
  capacities: { water: 2000 },
  description: '',
  organizationId: 'tbz',
} as unknown as import('@/api/backendApi').Vehicle

const renderDashboard = () => {
  const rootRoute = createRootRoute({ component: () => <Outlet /> })
  const indexRoute = createRoute({
    getParentRoute: () => rootRoute,
    path: '/',
    component: () => <VehicleDashboard vehicle={vehicle} />,
  })
  const router = createRouter({
    routeTree: rootRoute.addChildren([indexRoute]),
    history: createMemoryHistory({ initialEntries: ['/'] }),
  })
  const queryClient = new QueryClient({ defaultOptions: { queries: { retry: false } } })
  return render(
    <QueryClientProvider client={queryClient}>
      <RouterProvider router={router} />
    </QueryClientProvider>,
  )
}

const orgs = [
  { id: 'root', name: 'Green Ecolution', parentId: null, memberCount: 0 },
  { id: 'tbz', name: 'Betriebshof', parentId: 'root', memberCount: 0 },
]

describe('VehicleDashboard actions', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    listOrganizations.mockResolvedValue(orgs)
    getMe.mockResolvedValue({
      id: 'me',
      organization: orgs[0],
      roles: [{ id: 'r', name: 'Rolle', organizationId: 'root', permissions: ['vehicle:update'] }],
    })
  })

  it('renders no actions without vehicle:update', async () => {
    permissions.mockReturnValue(new Set(['vehicle:read']))
    renderDashboard()
    await screen.findByText('Fahrzeug: FL-GE 123')
    expect(screen.queryByText('Fahrzeug bearbeiten')).not.toBeInTheDocument()
    expect(screen.queryByLabelText('Weitere Aktionen')).not.toBeInTheDocument()
  })

  it('shows only the edit link when no other organization is reachable', async () => {
    permissions.mockReturnValue(new Set(['vehicle:update']))
    renderDashboard()
    expect(await screen.findByText('Fahrzeug bearbeiten')).toBeInTheDocument()
    expect(screen.queryByLabelText('Weitere Aktionen')).not.toBeInTheDocument()
  })

  it('moves the vehicle to the chosen organization', async () => {
    permissions.mockReturnValue(new Set(['vehicle:update', 'organization:read']))
    const transfer = vi.spyOn(vehicleApi, 'transferVehicle').mockResolvedValue(undefined)
    const user = userEvent.setup()
    renderDashboard()

    await user.click(await screen.findByLabelText('Weitere Aktionen'))
    await user.click(
      await screen.findByRole('menuitem', { name: 'In andere Organisation verschieben' }),
    )
    await user.click(screen.getByRole('combobox', { name: /zielorganisation/i }))
    await user.click(screen.getByRole('option', { name: 'Green Ecolution' }))
    await user.click(screen.getByRole('button', { name: 'Verschieben' }))

    await waitFor(() =>
      expect(transfer).toHaveBeenCalledWith({
        vehicleId: 'v-1',
        transferRequest: { organizationId: 'root' },
      }),
    )
  })
})
