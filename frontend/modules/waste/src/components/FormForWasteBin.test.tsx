import type { ReactNode } from 'react'
import { beforeEach, describe, expect, it, vi, type MockInstance } from 'vitest'
import { act, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { FormProvider } from 'react-hook-form'
import type { OrganizationResponse, UserResponse } from '@green-ecolution/backend-client'
import { organizationApi, userApi } from '@/api/backendApi'
import type { MapClickLngLat } from '@/components/map-gl/useMapClick'
import type { Permissions } from '@/lib/auth/permissions'
import useStore from '@/store/store'
import { wasteBinApi } from '../api/client'
import { wasteBinFixtures } from '../test/handlers'
import { renderInRouter } from '../test/renderInRouter'

const mapClick: { current?: (pos: MapClickLngLat) => void } = {}

vi.mock('@/components/map-gl/MapPreview', () => ({
  default: ({ children }: { children?: ReactNode }) => <div>{children}</div>,
}))
vi.mock('@/components/map-gl/useMapClick', () => ({
  useMapClick: (onClick: (pos: MapClickLngLat) => void) => {
    mapClick.current = onClick
  },
}))
vi.mock('@/components/map-gl/DraggableMarker', () => ({
  default: ({ onDragEnd }: { onDragEnd?: (pos: { lat: number; lng: number }) => void }) => (
    <button type="button" onClick={() => onDragEnd?.({ lat: 54.79, lng: 9.44 })}>
      Marker verschieben
    </button>
  ),
}))

const permissions = vi.fn((): Permissions => new Set<string>())
vi.mock('@/lib/auth/usePermissions', () => ({
  usePermissions: () => permissions(),
}))

vi.mock('@/lib/auth/authSessionContext', () => ({
  useAuthSession: () => ({ isAuthenticated: true }),
}))

const { default: FormForWasteBin } = await import('./FormForWasteBin')
const { useWasteBinForm, useWasteBinOrganizations } = await import('../hooks/useWasteBinForm')

const org = (id: string, name: string, parentId?: string) =>
  ({ id, name, parentId, memberCount: 0 }) as OrganizationResponse

const me = {
  id: 'user-1',
  organization: org('org-1', 'Stadt Flensburg'),
  roles: [{ id: 'role-1', organizationId: 'org-1', permissions: ['waste_bin:create'] }],
} as unknown as UserResponse

function CreateHarness() {
  const { form, mutate, isError, error } = useWasteBinForm('create', {
    initForm: { number: '' },
  })
  const organizations = useWasteBinOrganizations(form)
  return (
    <FormProvider {...form}>
      <FormForWasteBin
        organizations={organizations}
        displayError={isError}
        errorMessage={error?.message}
        onSubmit={(data) => mutate(data)}
      />
    </FormProvider>
  )
}

const fillMasterData = async (
  user: ReturnType<typeof userEvent.setup>,
  { depth = '120' }: { depth?: string } = {},
) => {
  await user.type(await screen.findByLabelText(/Behälternummer/), 'B-0100')
  await user.type(screen.getByLabelText(/Volumen/), '1100')
  await user.type(screen.getByLabelText(/Tiefe/), depth)
}

describe('FormForWasteBin', { timeout: 20_000 }, () => {
  let listOrganizations: MockInstance<typeof organizationApi.listOrganizations>

  beforeEach(() => {
    vi.restoreAllMocks()
    useStore.getState().clearAllFormDrafts()
    mapClick.current = undefined
    permissions.mockReturnValue(new Set(['waste_bin:create', 'organization:read']))
    vi.spyOn(userApi, 'getMe').mockResolvedValue(me)
    listOrganizations = vi
      .spyOn(organizationApi, 'listOrganizations')
      .mockResolvedValue([org('org-root', 'Wurzel'), org('org-1', 'Stadt Flensburg', 'org-root')])
  })

  it('reports every required field and sends nothing', async () => {
    const create = vi.spyOn(wasteBinApi, 'createWasteBin')
    const user = userEvent.setup()
    renderInRouter(<CreateHarness />)

    await user.click(await screen.findByRole('button', { name: /Speichern/ }))

    expect(await screen.findByText('Behälternummer ist erforderlich.')).toBeInTheDocument()
    expect(screen.getByText('Das Volumen muss eine Zahl sein.')).toBeInTheDocument()
    expect(screen.getByText('Die Tiefe muss eine ganze Zahl sein.')).toBeInTheDocument()
    expect(screen.getByText('Bitte den Standort auf der Karte wählen.')).toBeInTheDocument()
    expect(create).not.toHaveBeenCalled()
  })

  it('shows the domain message for a depth of 0', async () => {
    const user = userEvent.setup()
    renderInRouter(<CreateHarness />)

    await fillMasterData(user, { depth: '0' })
    await user.click(screen.getByRole('button', { name: /Speichern/ }))

    expect(await screen.findByText('Tiefe muss zwischen 1 und 1000 cm liegen.')).toBeInTheDocument()
  })

  it('creates the bin at the marker position in the own organization', async () => {
    const create = vi
      .spyOn(wasteBinApi, 'createWasteBin')
      .mockResolvedValue({ ...wasteBinFixtures[1], id: 'new-bin' })
    const user = userEvent.setup()
    renderInRouter(<CreateHarness />)

    await fillMasterData(user)
    act(() => mapClick.current?.({ lat: 54.78, lng: 9.43 }))
    await user.click(await screen.findByRole('button', { name: 'Marker verschieben' }))
    await user.click(screen.getByRole('button', { name: /Speichern/ }))

    await waitFor(() =>
      expect(create).toHaveBeenCalledWith({
        wasteBinCreateRequest: {
          number: 'B-0100',
          latitude: 54.79,
          longitude: 9.44,
          volumeLiters: 1100,
          depthCm: 120,
          organizationId: 'org-1',
        },
      }),
    )
  })

  it('offers the organization only when there is a choice', async () => {
    renderInRouter(<CreateHarness />)
    await screen.findByLabelText(/Behälternummer/)
    await waitFor(() => expect(listOrganizations).toHaveBeenCalled())
    // The role grants org-1 only; a subtree of one leaves nothing to pick.
    expect(screen.queryByText('Organisation')).not.toBeInTheDocument()
  })
})
