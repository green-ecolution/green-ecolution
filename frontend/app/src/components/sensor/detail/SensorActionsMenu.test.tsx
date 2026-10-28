/* eslint-disable @typescript-eslint/ban-ts-comment */
// @ts-nocheck - ad-hoc routes are not part of the generated route tree
import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { irrigationManifest } from '@green-ecolution/module-irrigation'
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
const textTarget = vi.fn(() => undefined as unknown)

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

vi.mock('./SensorActionsContext', () => ({
  useSensorActions: () => ({
    canAssign: true,
    currentTarget: undefined,
    textTarget: textTarget(),
    requestActivate: vi.fn(),
    requestReassign: vi.fn(),
    requestRemove: vi.fn(),
  }),
}))

const { default: SensorActionsMenu } = await import('./SensorActionsMenu')

const sensor = {
  id: 'eui-1',
  status: 'active',
  organizationId: 'tbz',
  target: null,
} as unknown as import('@/api/backendApi').Sensor

const renderMenu = async (subject = sensor) => {
  const queryClient = new QueryClient({ defaultOptions: { queries: { retry: false } } })
  const rootRoute = createRootRoute({ component: () => <Outlet /> })
  const indexRoute = createRoute({
    getParentRoute: () => rootRoute,
    path: '/',
    component: () => (
      <QueryClientProvider client={queryClient}>
        <SensorActionsMenu sensor={subject} />
      </QueryClientProvider>
    ),
  })
  const router = createRouter({
    routeTree: rootRoute.addChildren([indexRoute]),
    history: createMemoryHistory({ initialEntries: ['/'] }),
  })
  // preload route matches so the initial render below isn't left pending
  await router.load()
  return render(<RouterProvider router={router} />)
}

describe('SensorActionsMenu', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    permissions.mockReturnValue(new Set<string>())
    textTarget.mockReturnValue(undefined)
  })

  it('hides the trigger without sensor:update or sensor:delete', async () => {
    permissions.mockReturnValue(new Set(['sensor:read']))
    await renderMenu()
    expect(screen.queryByRole('button', { name: 'Aktionen' })).not.toBeInTheDocument()
  })

  it('shows the trigger with sensor:update', async () => {
    permissions.mockReturnValue(new Set(['sensor:update']))
    await renderMenu()
    expect(await screen.findByRole('button', { name: 'Aktionen' })).toBeInTheDocument()
  })

  it('shows the trigger with only sensor:delete', async () => {
    permissions.mockReturnValue(new Set(['sensor:delete']))
    await renderMenu()
    expect(await screen.findByRole('button', { name: 'Aktionen' })).toBeInTheDocument()
  })

  it('words the link actions neutrally without a target to name', async () => {
    permissions.mockReturnValue(new Set(['sensor:update']))
    await renderMenu()
    await userEvent.click(await screen.findByRole('button', { name: 'Aktionen' }))
    expect(await screen.findByRole('menuitem', { name: 'Verknüpfung ändern' })).toBeInTheDocument()
    expect(screen.getByRole('menuitem', { name: 'Verknüpfung aufheben' })).toBeInTheDocument()
  })

  it('takes the wording of the tree target', async () => {
    permissions.mockReturnValue(new Set(['sensor:update']))
    textTarget.mockReturnValue(irrigationManifest.targets?.[0])
    await renderMenu()
    await userEvent.click(await screen.findByRole('button', { name: 'Aktionen' }))
    expect(
      await screen.findByRole('menuitem', { name: 'Anderen Baum zuweisen' }),
    ).toBeInTheDocument()
    expect(screen.getByRole('menuitem', { name: 'Baumverknüpfung aufheben' })).toBeInTheDocument()
  })
})

describe('SensorActionsMenu transfer', () => {
  const orgs = [
    { id: 'root', name: 'Green Ecolution', parentId: null, memberCount: 0 },
    { id: 'tbz', name: 'Betriebshof', parentId: 'root', memberCount: 0 },
  ]

  beforeEach(() => {
    vi.clearAllMocks()
    textTarget.mockReturnValue(undefined)
    permissions.mockReturnValue(new Set(['sensor:update', 'organization:read']))
    listOrganizations.mockResolvedValue(orgs)
    getMe.mockResolvedValue({
      id: 'me',
      organization: orgs[0],
      roles: [{ id: 'r', name: 'Rolle', organizationId: 'root', permissions: ['sensor:update'] }],
    })
  })

  it('moves an unlinked sensor to the chosen organization', async () => {
    const { sensorApi } = await import('@/api/backendApi')
    const transfer = vi.spyOn(sensorApi, 'transferSensor').mockResolvedValue(undefined)
    const user = userEvent.setup()
    await renderMenu()

    await user.click(await screen.findByRole('button', { name: 'Aktionen' }))
    await user.click(
      await screen.findByRole('menuitem', { name: 'In andere Organisation verschieben' }),
    )
    await user.click(screen.getByRole('combobox', { name: /zielorganisation/i }))
    await user.click(screen.getByRole('option', { name: 'Green Ecolution' }))
    await user.click(screen.getByRole('button', { name: 'Verschieben' }))

    await waitFor(() =>
      expect(transfer).toHaveBeenCalledWith({
        sensorId: 'eui-1',
        transferRequest: { organizationId: 'root' },
      }),
    )
  })

  it('explains why a bound sensor cannot move alone', async () => {
    const user = userEvent.setup()
    await renderMenu({ ...sensor, target: { module: 'irrigation', kind: 'tree', id: 't-1' } })

    await user.click(await screen.findByRole('button', { name: 'Aktionen' }))
    const item = await screen.findByRole('menuitem', {
      name: /In andere Organisation verschieben/,
    })
    expect(item).toHaveAttribute('aria-disabled', 'true')
    expect(item).toHaveTextContent('Zieht mit seinem verknüpften Objekt um.')
  })
})
