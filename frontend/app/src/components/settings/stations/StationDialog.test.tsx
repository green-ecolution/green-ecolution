import type { ComponentProps, ReactNode } from 'react'
import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import type { ModuleCapacityKind } from '@/modules/types'
import { manifestOf } from '@/test/moduleFixtures'

const water = manifestOf('irrigation').capacities![0]
const cargo = manifestOf('waste').capacities![0]
const offered = vi.fn((): ModuleCapacityKind[] => [water, cargo])
const offeredFor = vi.fn()
vi.mock('@/modules/capacities', async (importOriginal) => ({
  ...(await importOriginal<typeof import('@/modules/capacities')>()),
  useCapacityKinds: (organizationId: string | undefined) => {
    offeredFor(organizationId)
    return offered()
  },
}))

const { default: StationDialog } = await import('./StationDialog')

vi.mock('@/components/map-gl/MapPreview', () => ({
  default: ({ children }: { children?: ReactNode }) => (
    <div data-testid="map-preview">{children}</div>
  ),
}))

vi.mock('@/components/map-gl/DraggableMarker', () => ({
  default: () => <div data-testid="draggable-marker" />,
}))

describe('StationDialog', { timeout: 20_000 }, () => {
  const onSubmit = vi.fn()

  beforeEach(() => {
    onSubmit.mockClear()
    offered.mockReturnValue([water, cargo])
  })

  const editStation = (replenishes: string[]) => (
    <StationDialog
      open
      mode="edit"
      onOpenChange={vi.fn()}
      onSubmit={onSubmit}
      organizationId="org-2"
      station={{
        id: 's-1',
        name: 'Betriebshof',
        lat: 54.7,
        lon: 9.4,
        isDefault: false,
        organizationId: 'org-2',
        replenishes,
      }}
    />
  )

  it("offers the actions of the modules active in the station's organization", () => {
    offered.mockReturnValue([cargo])
    render(editStation([]))

    expect(offeredFor).toHaveBeenCalledWith('org-2')
    expect(screen.getByRole('checkbox', { name: 'Laderaum entladen' })).toBeInTheDocument()
    expect(screen.queryByRole('checkbox', { name: 'Wasser auffüllen' })).not.toBeInTheDocument()
  })

  it('keeps a stored kind that is not offered', async () => {
    const user = userEvent.setup()
    offered.mockReturnValue([cargo])
    render(editStation(['water']))

    await user.click(screen.getByRole('checkbox', { name: 'Laderaum entladen' }))
    await user.click(screen.getByRole('button', { name: 'Speichern' }))

    expect(onSubmit).toHaveBeenCalledWith(
      expect.objectContaining({ replenishes: ['water', 'cargo_volume'] }),
    )
  })

  it('shows only the hint without an offered kind', () => {
    offered.mockReturnValue([])
    render(editStation([]))

    expect(screen.queryByRole('checkbox')).not.toBeInTheDocument()
    expect(
      screen.getByText('Ohne Auswahl dient die Station nur als Start und Ziel einer Tour.'),
    ).toBeInTheDocument()
  })

  const renderDialog = (props: Partial<ComponentProps<typeof StationDialog>> = {}) =>
    render(
      <StationDialog
        open
        mode="create"
        organizationId="org-1"
        onOpenChange={vi.fn()}
        onSubmit={onSubmit}
        initialPosition={{ lat: 54.78, lng: 9.43 }}
        {...props}
      />,
    )

  const fillAndSubmit = async ({ name }: { name: string }) => {
    const user = userEvent.setup()
    await user.type(screen.getByLabelText('Name'), name)
    await user.click(screen.getByRole('button', { name: 'Speichern' }))
  }

  it('has no organization field, the page decides where a station is created', () => {
    renderDialog({ mode: 'create', organizationId: 'amt' })
    expect(screen.queryByLabelText(/Organisation/)).not.toBeInTheDocument()
  })

  it('submits the given organization on create', async () => {
    renderDialog({ mode: 'create', organizationId: 'amt' })
    await fillAndSubmit({ name: 'Depot West' })
    expect(onSubmit).toHaveBeenCalledWith(expect.objectContaining({ organizationId: 'amt' }))
  })

  it('requires a name', async () => {
    const user = userEvent.setup()
    renderDialog()
    await user.click(screen.getByRole('button', { name: 'Speichern' }))
    expect(await screen.findByText('Bitte einen Namen angeben.')).toBeInTheDocument()
    expect(onSubmit).not.toHaveBeenCalled()
  })

  it('submits the chosen capacity kinds', async () => {
    const user = userEvent.setup()
    renderDialog()
    await user.type(screen.getByLabelText('Name'), 'Wertstoffhof')
    await user.click(screen.getByRole('checkbox', { name: 'Laderaum entladen' }))
    await user.click(screen.getByRole('button', { name: 'Speichern' }))
    expect(onSubmit).toHaveBeenCalledWith({
      name: 'Wertstoffhof',
      lat: 54.78,
      lon: 9.43,
      organizationId: 'org-1',
      replenishes: ['cargo_volume'],
    })
  })

  it('prefills the station when editing and leaves the organization out', async () => {
    const user = userEvent.setup()
    render(
      <StationDialog
        open
        mode="edit"
        onOpenChange={vi.fn()}
        onSubmit={onSubmit}
        organizationId="org-2"
        station={{
          id: 's-1',
          name: 'Betriebshof',
          lat: 54.7,
          lon: 9.4,
          isDefault: false,
          organizationId: 'org-2',
          replenishes: ['water'],
        }}
      />,
    )
    expect(screen.getByLabelText('Name')).toHaveValue('Betriebshof')
    expect(screen.getByRole('checkbox', { name: 'Wasser auffüllen' })).toBeChecked()
    await user.click(screen.getByRole('button', { name: 'Speichern' }))
    expect(onSubmit).toHaveBeenCalledWith({
      name: 'Betriebshof',
      lat: 54.7,
      lon: 9.4,
      replenishes: ['water'],
    })
  })
})
