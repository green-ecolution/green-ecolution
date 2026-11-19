import { screen, waitFor, within } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import type { StartPointResponse } from '@/api/backendApi'
import type { Permissions } from '@/lib/auth/permissions'
import { render } from '@/test/utils'
import StationList from './StationList'

const permissions = vi.hoisted(
  (): { current: Set<string>; byOrg: Partial<Record<string, Permissions>> } => ({
    current: new Set<string>(),
    byOrg: {},
  }),
)

vi.mock('@/lib/auth/usePermissions', () => ({
  usePermissions: (): Permissions => permissions.current,
}))
vi.mock('@/lib/auth/usePermissionsIn', () => ({
  usePermissionsResolver: () => (orgId: string | null) =>
    (orgId === null ? undefined : permissions.byOrg[orgId]) ?? new Set<string>(),
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

const stations: StartPointResponse[] = [
  {
    id: 's-1',
    name: 'Betriebshof',
    lat: 54.7,
    lon: 9.4,
    isDefault: true,
    organizationId: 'org-1',
    replenishes: ['water'],
  },
  {
    id: 's-2',
    name: 'Wertstoffhof',
    lat: 54.8,
    lon: 9.5,
    isDefault: false,
    organizationId: 'org-2',
    replenishes: ['water', 'cargo_volume'],
  },
]
const organizations = [
  { id: 'org-1', name: 'Stadt Flensburg', memberCount: 0 },
  { id: 'org-2', name: 'TBZ', memberCount: 0 },
]

const renderList = (handlers = {}) => {
  const props = {
    onEdit: vi.fn(),
    onSetDefault: vi.fn(),
    onDelete: vi.fn(),
    onTransfer: vi.fn(),
    ...handlers,
  }
  render(<StationList stations={stations} organizations={organizations} {...props} />)
  return props
}

const grantInEvery = (rights: string[]) => {
  permissions.byOrg = { 'org-1': new Set(rights), 'org-2': new Set(rights) }
}

describe('StationList', () => {
  beforeEach(() => {
    permissions.current = new Set(['start_point:read'])
    grantInEvery(['start_point:read'])
  })

  it('shows name, organization, capacity kinds and the default marker', () => {
    renderList()
    expect(screen.getByText('Betriebshof')).toBeInTheDocument()
    expect(screen.getByText('TBZ')).toBeInTheDocument()
    expect(screen.getAllByText('Wasser')).toHaveLength(2)
    expect(screen.getByText('Laderaum')).toBeInTheDocument()
    expect(screen.getAllByText('Standard')).toHaveLength(1)
  })

  it('offers no actions without write permissions', () => {
    renderList()
    expect(screen.queryByRole('button', { name: /als standard setzen/i })).not.toBeInTheDocument()
    expect(screen.queryByRole('button', { name: /löschen/i })).not.toBeInTheDocument()
    expect(screen.queryByRole('button', { name: /bearbeiten/i })).not.toBeInTheDocument()
  })

  it('sets a station as default only for non-default stations', async () => {
    grantInEvery(['start_point:read', 'start_point:update'])
    const user = userEvent.setup()
    const { onSetDefault } = renderList()
    const buttons = screen.getAllByRole('button', { name: /als standard setzen/i })
    expect(buttons).toHaveLength(1)
    await user.click(buttons[0])
    expect(onSetDefault).toHaveBeenCalledWith(stations[1])
  })

  it('asks for confirmation before deleting', async () => {
    grantInEvery(['start_point:read', 'start_point:delete'])
    const user = userEvent.setup()
    const { onDelete } = renderList()
    await user.click(screen.getByRole('button', { name: 'Station Wertstoffhof löschen' }))
    expect(onDelete).not.toHaveBeenCalled()
    const dialog = await screen.findByRole('alertdialog')
    await user.click(within(dialog).getByRole('button', { name: 'Löschen' }))
    expect(onDelete).toHaveBeenCalledWith(stations[1])
  })

  it('offers edit and delete only for stations of organizations with the right', () => {
    permissions.byOrg = {
      'org-1': new Set(['start_point:read']),
      'org-2': new Set(['start_point:read', 'start_point:update', 'start_point:delete']),
    }
    renderList()
    expect(
      screen.getByRole('button', { name: 'Station Wertstoffhof bearbeiten' }),
    ).toBeInTheDocument()
    expect(screen.getByRole('button', { name: 'Station Wertstoffhof löschen' })).toBeInTheDocument()
    expect(
      screen.queryByRole('button', { name: 'Station Betriebshof bearbeiten' }),
    ).not.toBeInTheDocument()
    expect(
      screen.queryByRole('button', { name: 'Station Betriebshof löschen' }),
    ).not.toBeInTheDocument()
  })
})

describe('StationList transfer', () => {
  const tree = [
    { id: 'root', name: 'Green Ecolution', parentId: null, memberCount: 0 },
    { id: 'org-1', name: 'Stadt Flensburg', parentId: 'root', memberCount: 0 },
    { id: 'org-2', name: 'TBZ', parentId: 'root', memberCount: 0 },
  ]

  beforeEach(() => {
    vi.clearAllMocks()
    permissions.current = new Set(['start_point:read', 'start_point:update', 'organization:read'])
    grantInEvery(['start_point:read', 'start_point:update'])
    listOrganizations.mockResolvedValue(tree)
    getMe.mockResolvedValue({
      id: 'me',
      organization: tree[0],
      roles: [
        { id: 'r', name: 'Rolle', organizationId: 'root', permissions: ['start_point:update'] },
      ],
    })
  })

  it('moves a non-default station to the chosen organization', async () => {
    const user = userEvent.setup()
    const { onTransfer } = renderList()

    await user.click(
      await screen.findByRole('button', { name: 'Station Wertstoffhof verschieben' }),
    )
    await user.click(screen.getByRole('combobox', { name: /zielorganisation/i }))
    await user.click(screen.getByRole('option', { name: 'Stadt Flensburg' }))
    await user.click(screen.getByRole('button', { name: 'Verschieben' }))

    expect(onTransfer).toHaveBeenCalledWith(stations[1], 'org-1')
  })

  it('never offers to move the default station', async () => {
    renderList()

    await screen.findByRole('button', { name: 'Station Wertstoffhof verschieben' })
    await waitFor(() => expect(listOrganizations).toHaveBeenCalled())
    expect(
      screen.queryByRole('button', { name: 'Station Betriebshof verschieben' }),
    ).not.toBeInTheDocument()
  })
})
