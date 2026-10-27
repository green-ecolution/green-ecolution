import type { ReactNode } from 'react'
import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import StationDialog from './StationDialog'

vi.mock('@/components/map-gl/MapPreview', () => ({
  default: ({ children }: { children?: ReactNode }) => (
    <div data-testid="map-preview">{children}</div>
  ),
}))

vi.mock('@/components/map-gl/DraggableMarker', () => ({
  default: () => <div data-testid="draggable-marker" />,
}))

const orgs = [
  { id: 'org-1', name: 'Stadt Flensburg', memberCount: 0 },
  { id: 'org-2', name: 'TBZ', memberCount: 0 },
]

describe('StationDialog', { timeout: 20_000 }, () => {
  const onSubmit = vi.fn()

  beforeEach(() => onSubmit.mockClear())

  it('requires a name', async () => {
    const user = userEvent.setup()
    render(
      <StationDialog
        open
        mode="create"
        onOpenChange={vi.fn()}
        onSubmit={onSubmit}
        organizations={orgs}
        initialPosition={{ lat: 54.78, lng: 9.43 }}
      />,
    )
    await user.click(screen.getByRole('button', { name: 'Speichern' }))
    expect(await screen.findByText('Bitte einen Namen angeben.')).toBeInTheDocument()
    expect(onSubmit).not.toHaveBeenCalled()
  })

  it('submits the chosen capacity kinds', async () => {
    const user = userEvent.setup()
    render(
      <StationDialog
        open
        mode="create"
        onOpenChange={vi.fn()}
        onSubmit={onSubmit}
        organizations={orgs}
        initialPosition={{ lat: 54.78, lng: 9.43 }}
      />,
    )
    await user.type(screen.getByLabelText('Name'), 'Wertstoffhof')
    await user.click(screen.getByRole('checkbox', { name: 'Laderaum' }))
    await user.click(screen.getByRole('button', { name: 'Speichern' }))
    expect(onSubmit).toHaveBeenCalledWith({
      name: 'Wertstoffhof',
      lat: 54.78,
      lon: 9.43,
      organizationId: orgs[0].id,
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
        organizations={orgs}
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
    expect(screen.getByRole('checkbox', { name: 'Wasser' })).toBeChecked()
    await user.click(screen.getByRole('button', { name: 'Speichern' }))
    expect(onSubmit).toHaveBeenCalledWith({
      name: 'Betriebshof',
      lat: 54.7,
      lon: 9.4,
      replenishes: ['water'],
    })
  })
})
