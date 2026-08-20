import { describe, it, expect, vi } from 'vitest'
import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import ForeignTreeDialog from './ForeignTreeDialog'

const baseProps = {
  open: true,
  onOpenChange: vi.fn(),
  organizationName: 'Betriebshof',
  blockedReason: 'In dieser Organisation darfst du keine Gruppe anlegen.',
  selectedTreeCount: 0,
  onConfirm: vi.fn(),
}

describe('ForeignTreeDialog', () => {
  it('offers the switch and names the organization', () => {
    render(<ForeignTreeDialog {...baseProps} canSwitch />)

    expect(screen.getByRole('heading', { name: /organisation wechseln/i })).toBeInTheDocument()
    expect(screen.getByText(/gehört der Organisation Betriebshof/i)).toBeInTheDocument()
    expect(screen.getByRole('button', { name: /wechseln/i })).toBeInTheDocument()
  })

  it('warns about the trees a switch would discard', () => {
    render(<ForeignTreeDialog {...baseProps} canSwitch selectedTreeCount={3} />)

    expect(screen.getByText(/ausgewählten Bäume \(3\) werden dabei verworfen/i)).toBeInTheDocument()
  })

  it('says nothing about discarding when nothing is selected', () => {
    render(<ForeignTreeDialog {...baseProps} canSwitch />)

    expect(screen.queryByText(/verworfen/i)).not.toBeInTheDocument()
  })

  it('states the reason instead of offering a switch when blocked', () => {
    render(<ForeignTreeDialog {...baseProps} canSwitch={false} />)

    expect(screen.getByRole('heading', { name: /nicht auswählbar/i })).toBeInTheDocument()
    expect(screen.getByText(/darfst du keine Gruppe anlegen/i)).toBeInTheDocument()
    expect(screen.queryByRole('button', { name: /wechseln/i })).not.toBeInTheDocument()
  })

  it('falls back to a generic owner when the name is unknown', () => {
    render(<ForeignTreeDialog {...baseProps} canSwitch organizationName={undefined} />)

    expect(screen.getByText(/gehört einer anderen Organisation/i)).toBeInTheDocument()
  })

  it('confirms the switch', async () => {
    const onConfirm = vi.fn()
    render(<ForeignTreeDialog {...baseProps} canSwitch onConfirm={onConfirm} />)

    await userEvent.click(screen.getByRole('button', { name: /wechseln/i }))

    expect(onConfirm).toHaveBeenCalledTimes(1)
  })
})
