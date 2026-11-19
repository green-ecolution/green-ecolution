import type { ComponentProps, ReactNode } from 'react'
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

describe('StationDialog', { timeout: 20_000 }, () => {
  const onSubmit = vi.fn()

  beforeEach(() => onSubmit.mockClear())

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
