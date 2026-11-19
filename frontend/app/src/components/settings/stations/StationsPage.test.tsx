import { screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import type { OrganizationResponse, StartPointResponse } from '@/api/backendApi'
import type { Permissions } from '@/lib/auth/permissions'
import { render } from '@/test/utils'
import type { StationPayload } from './StationDialog'
import StationsPage from './StationsPage'

const ORGS: OrganizationResponse[] = [
  { id: 'root', name: 'Instanz', memberCount: 0 },
  { id: 'amt', name: 'Grünflächenamt', memberCount: 3, parentId: 'root' },
  { id: 'nord', name: 'Stadtgärtnerei Nord', memberCount: 4, parentId: 'amt' },
  { id: 'fremd', name: 'Fremdamt', memberCount: 1, parentId: 'root' },
]

const station = (id: string, name: string, organizationId: string): StartPointResponse => ({
  id,
  name,
  lat: 54.7,
  lon: 9.4,
  isDefault: false,
  organizationId,
  replenishes: ['water'],
})

const state = vi.hoisted(
  (): {
    perms: Permissions
    organizations: OrganizationResponse[]
    stations: StartPointResponse[] | null
    dialogProps: { organizationId: string; onSubmit: (payload: StationPayload) => void } | null
  } => ({ perms: 'unrestricted', organizations: [], stations: [], dialogProps: null }),
)

vi.mock('../organization/useSettingsOrganization', () => ({
  useSettingsOrganization: () => ({
    orgId: 'amt',
    organizations: state.organizations,
    perms: state.perms,
    can: (required: readonly string[]) =>
      state.perms === 'unrestricted' ||
      required.length === 0 ||
      required.some((p) => (state.perms as Set<string>).has(p)),
  }),
}))

vi.mock('@/api/queries', async (importOriginal) => ({
  ...(await importOriginal<typeof import('@/api/queries')>()),
  routingStartPointsQuery: () => ({
    queryKey: ['test-start-points'],
    queryFn: () => Promise.resolve(state.stations),
  }),
  userQueries: {
    mapView: () => ({ queryKey: ['test-map-view'], queryFn: () => Promise.resolve(undefined) }),
  },
}))

const create = vi.fn()
vi.mock('@/hooks/useStationMutations', () => ({
  useStationMutations: () => ({
    create: { mutate: create, isPending: false },
    update: { mutate: vi.fn(), isPending: false },
    remove: { mutate: vi.fn() },
    setDefault: { mutate: vi.fn() },
    transfer: { mutate: vi.fn() },
  }),
}))

vi.mock('./StationDialog', () => ({
  default: (props: { organizationId: string; onSubmit: (payload: StationPayload) => void }) => {
    state.dialogProps = props
    return null
  },
}))

vi.mock('./StationList', () => ({
  default: ({ stations }: { stations: StartPointResponse[] }) => (
    <ul>
      {stations.map((s) => (
        <li key={s.id}>{s.name}</li>
      ))}
    </ul>
  ),
}))

// Like the real dialog, the mock adds the organization it was given on create.
const submitDialog = (payload: StationPayload) =>
  state.dialogProps?.onSubmit({ ...payload, organizationId: state.dialogProps.organizationId })

describe('StationsPage', () => {
  beforeEach(() => {
    create.mockClear()
    state.perms = 'unrestricted'
    state.organizations = ORGS
    state.stations = [
      station('s1', 'Depot Amt', 'amt'),
      station('s2', 'Depot Nord', 'nord'),
      station('s3', 'Depot Fremd', 'fremd'),
    ]
  })

  it('lists the stations of the selected organization and its sub-organizations', async () => {
    render(<StationsPage />)
    expect(await screen.findByText('Depot Amt')).toBeInTheDocument()
    expect(screen.getByText('Depot Nord')).toBeInTheDocument()
    expect(screen.queryByText('Depot Fremd')).not.toBeInTheDocument()
  })

  it('creates a station in the selected organization', async () => {
    render(<StationsPage />)
    await userEvent.click(await screen.findByRole('button', { name: /Station anlegen/ }))
    submitDialog({ name: 'Depot West', lat: 54.79, lon: 9.43, replenishes: [] })
    expect(create).toHaveBeenCalledWith(
      expect.objectContaining({ name: 'Depot West', organizationId: 'amt' }),
      expect.anything(),
    )
  })

  it('hides the create button without start_point:create in this organization', async () => {
    state.perms = new Set(['start_point:read'])
    render(<StationsPage />)
    expect(await screen.findByText('Depot Amt')).toBeInTheDocument()
    expect(screen.queryByRole('button', { name: /Station anlegen/ })).not.toBeInTheDocument()
  })

  // Without organization:read the context offers the reachable organizations.
  it('lists the stations for a role that holds only start_point:read', async () => {
    state.perms = new Set(['start_point:read'])
    state.organizations = [
      { id: 'amt', name: 'Grünflächenamt', memberCount: 0, parentId: 'root' },
      { id: 'nord', name: 'Stadtgärtnerei Nord', memberCount: 0, parentId: 'amt' },
    ]
    render(<StationsPage />)
    expect(await screen.findByText('Depot Amt')).toBeInTheDocument()
    expect(screen.getByText('Depot Nord')).toBeInTheDocument()
  })

  it('shows the routing hint when routing is switched off', async () => {
    state.stations = null
    render(<StationsPage />)
    expect(await screen.findByText(/Routing/)).toBeInTheDocument()
  })
})
