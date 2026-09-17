import { describe, it, expect, vi } from 'vitest'
import { render, screen, fireEvent } from '@testing-library/react'
import ActiveFilterChips from './ActiveFilterChips'

describe('ActiveFilterChips', () => {
  it('renders nothing but the result label when no chip is set', () => {
    render(<ActiveFilterChips chips={[]} onReset={vi.fn()} resultLabel="248 Bäume" />)

    expect(screen.getByText('248 Bäume')).toBeInTheDocument()
    expect(screen.queryByRole('button', { name: /zurücksetzen/i })).not.toBeInTheDocument()
  })

  it('removes exactly the chip that was clicked', () => {
    const removeCritical = vi.fn()
    const removeUnknown = vi.fn()
    render(
      <ActiveFilterChips
        chips={[
          { id: 'bad', label: 'Kritisch', onRemove: removeCritical },
          { id: 'unknown', label: 'Unbekannt', onRemove: removeUnknown },
        ]}
        onReset={vi.fn()}
        resultLabel="17 von 248 Bäumen"
      />,
    )

    fireEvent.click(screen.getByRole('button', { name: 'Filter Kritisch entfernen' }))

    expect(removeCritical).toHaveBeenCalledOnce()
    expect(removeUnknown).not.toHaveBeenCalled()
  })

  it('offers a reset once at least one chip is set', () => {
    const onReset = vi.fn()
    render(
      <ActiveFilterChips
        chips={[{ id: 'bad', label: 'Kritisch', onRemove: vi.fn() }]}
        onReset={onReset}
        resultLabel="17 von 248 Bäumen"
      />,
    )

    fireEvent.click(screen.getByRole('button', { name: /zurücksetzen/i }))
    expect(onReset).toHaveBeenCalledOnce()
  })
})
