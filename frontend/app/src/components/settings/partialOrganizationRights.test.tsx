import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import type { ReactNode } from 'react'
import { render, screen } from '@testing-library/react'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import type {
  OrganizationResponse,
  ReachableOrganizationResponse,
  RoleResponse,
  StartPointResponse,
} from '@green-ecolution/backend-client'
import {
  infoQueries,
  organizationQueries,
  routingStartPointsQuery,
  userQueries,
} from '@/api/queries'

// Organization:read in Grünflächenamt only; in the sibling Betriebshof the user
// holds start_point:read, so GET /organizations does not list Betriebshof.
const VISIBLE: OrganizationResponse[] = [
  { id: 'amt', name: 'Grünflächenamt', parentId: 'root', memberCount: 3 },
]
const REACHABLE: ReachableOrganizationResponse[] = [
  { id: 'amt', name: 'Grünflächenamt', parentId: 'root' },
  { id: 'betrieb', name: 'Betriebshof', parentId: 'root' },
  { id: 'west', name: 'Betriebshof West', parentId: 'betrieb' },
]
const ROLES = [
  { id: 'r1', name: 'Lesen', organizationId: 'amt', permissions: ['organization:read'] },
  { id: 'r2', name: 'Routen', organizationId: 'betrieb', permissions: ['start_point:read'] },
] as RoleResponse[]

const station = (id: string, name: string, organizationId: string): StartPointResponse => ({
  id,
  name,
  lat: 54.7,
  lon: 9.4,
  isDefault: false,
  organizationId,
  replenishes: ['water'],
})

vi.mock('@tanstack/react-router', () => ({
  Link: ({
    to,
    params,
    children,
  }: {
    to: string
    params?: Record<string, string>
    children: ReactNode
  }) => (
    <a
      href={Object.entries(params ?? {}).reduce(
        (path, [key, value]) => path.replace(`$${key}`, value),
        to,
      )}
    >
      {children}
    </a>
  ),
  useLocation: () => ({ pathname: '/settings/organization/betrieb/stations' }),
  useNavigate: () => vi.fn(),
  getRouteApi: () => ({ useParams: () => ({ orgId: 'betrieb' }) }),
}))
vi.mock('@/lib/auth/runtimeConfig', () => ({ readAuthBypass: () => false }))
vi.mock('@/lib/auth/scope/effectiveRoles', async (importOriginal) => ({
  ...(await importOriginal<typeof import('@/lib/auth/scope/effectiveRoles')>()),
  useEffectiveRoles: () => ROLES,
}))
vi.mock('@/lib/auth/scope/settingsOrganization', () => ({
  useSettingsHomeOrganizationId: () => 'betrieb',
}))
vi.mock('@/lib/auth/scope/adminMode', () => ({ useAdminMode: () => false }))
vi.mock('@/hooks/useStationMutations', () => ({
  useStationMutations: () => ({
    create: { mutate: vi.fn(), isPending: false },
    update: { mutate: vi.fn(), isPending: false },
    remove: { mutate: vi.fn() },
    setDefault: { mutate: vi.fn() },
    transfer: { mutate: vi.fn() },
  }),
}))
vi.mock('./stations/StationDialog', () => ({ default: () => null }))

const { default: SettingsLayout } = await import('./SettingsLayout')
const { default: StationsPage } = await import('./stations/StationsPage')

// jsdom applies no Tailwind; on a narrow screen only the picker above the strip shows.
let viewport: HTMLStyleElement | null = null
afterEach(() => viewport?.remove())

describe('settings for an organization without organization:read', () => {
  beforeEach(() => {
    viewport = document.createElement('style')
    viewport.textContent = '.hidden { display: none }'
    document.head.append(viewport)
  })

  it('names the organization and lists the stations of its subtree', async () => {
    const queryClient = new QueryClient({
      defaultOptions: { queries: { staleTime: Infinity, retry: false } },
    })
    queryClient.setQueryData(organizationQueries.list().queryKey, VISIBLE)
    queryClient.setQueryData(userQueries.myOrganizations().queryKey, REACHABLE)
    queryClient.setQueryData(userQueries.mapView().queryKey, {
      center: [54.7, 9.4],
    })
    queryClient.setQueryData(infoQueries.services().queryKey, {
      items: [{ name: 'routing', enabled: true }],
    } as never)
    queryClient.setQueryData(routingStartPointsQuery().queryKey, [
      station('s1', 'Depot Amt', 'amt'),
      station('s2', 'Depot Betriebshof', 'betrieb'),
      station('s3', 'Depot West', 'west'),
    ])

    render(
      <QueryClientProvider client={queryClient}>
        <SettingsLayout>
          <StationsPage />
        </SettingsLayout>
      </QueryClientProvider>,
    )

    expect(
      screen.getByRole('button', { name: 'Organisation wechseln, aktuell Betriebshof' }),
    ).toBeInTheDocument()
    expect(await screen.findByText('Depot Betriebshof')).toBeInTheDocument()
    expect(screen.getByText('Depot West')).toBeInTheDocument()
    expect(screen.queryByText('Depot Amt')).not.toBeInTheDocument()
  })
})
