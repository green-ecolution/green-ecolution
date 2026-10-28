import { describe, it, expect, vi } from 'vitest'
import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import TransferOrganizationDialog from './TransferOrganizationDialog'

const organizations = [
  { id: 'tbz', name: 'Betriebshof', memberCount: 0 },
  { id: 'extern', name: 'Extern A', memberCount: 0 },
]

const renderDialog = (
  overrides: Partial<React.ComponentProps<typeof TransferOrganizationDialog>> = {},
) => {
  const props = {
    open: true,
    onOpenChange: vi.fn(),
    title: 'Bewässerungsgruppe verschieben',
    description: 'Die Bäume der Gruppe und ihre Sensoren ziehen mit.',
    organizations,
    onConfirm: vi.fn(),
    ...overrides,
  }
  render(<TransferOrganizationDialog {...props} />)
  return props
}

describe('TransferOrganizationDialog', () => {
  it('shows the caller texts and keeps confirm disabled until an organization is chosen', () => {
    renderDialog()

    expect(screen.getByRole('heading', { name: 'Bewässerungsgruppe verschieben' })).toBeVisible()
    expect(screen.getByText('Die Bäume der Gruppe und ihre Sensoren ziehen mit.')).toBeVisible()
    expect(screen.getByRole('button', { name: 'Verschieben' })).toBeDisabled()
  })

  it('confirms with the chosen organization', async () => {
    const user = userEvent.setup()
    const { onConfirm } = renderDialog()

    await user.click(screen.getByRole('combobox', { name: /zielorganisation/i }))
    await user.click(screen.getByRole('option', { name: 'Extern A' }))
    await user.click(screen.getByRole('button', { name: 'Verschieben' }))

    expect(onConfirm).toHaveBeenCalledWith('extern')
  })

  it('blocks the buttons while the transfer runs', () => {
    renderDialog({ pending: true })

    expect(screen.getByRole('button', { name: 'Wird verschoben …' })).toBeDisabled()
    expect(screen.getByRole('button', { name: 'Abbrechen' })).toBeDisabled()
  })
})

describe('TransferOrganizationDialog reopening', () => {
  it('forgets the previous choice once closed by the caller', async () => {
    const user = userEvent.setup()
    const props = {
      onOpenChange: vi.fn(),
      title: 'Verschieben',
      description: 'Beschreibung',
      organizations,
      onConfirm: vi.fn(),
    }
    const { rerender } = render(<TransferOrganizationDialog {...props} open />)
    await user.click(screen.getByRole('combobox', { name: /zielorganisation/i }))
    await user.click(screen.getByRole('option', { name: 'Extern A' }))
    expect(screen.getByRole('button', { name: 'Verschieben' })).toBeEnabled()

    rerender(<TransferOrganizationDialog {...props} open={false} />)
    rerender(<TransferOrganizationDialog {...props} open />)

    expect(screen.getByRole('button', { name: 'Verschieben' })).toBeDisabled()
  })
})
