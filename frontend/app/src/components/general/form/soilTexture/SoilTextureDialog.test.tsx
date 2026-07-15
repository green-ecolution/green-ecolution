import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen, fireEvent } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { SoilCondition } from '@green-ecolution/backend-client'
import SoilTextureDialog from './SoilTextureDialog'

describe('SoilTextureDialog', () => {
  const mockOnOpenChange = vi.fn()
  const mockOnApply = vi.fn()

  beforeEach(() => {
    vi.clearAllMocks()
  })

  const renderDialog = (initialCondition: SoilCondition = SoilCondition.Sl3) =>
    render(
      <SoilTextureDialog
        open
        onOpenChange={mockOnOpenChange}
        initialCondition={initialCondition}
        onApply={mockOnApply}
      />,
    )

  it('initializes the fields from the current selection midpoint', () => {
    renderDialog(SoilCondition.Sl3)
    expect(screen.getByRole('spinbutton', { name: /sand/i })).toHaveValue(65)
    expect(screen.getByRole('spinbutton', { name: /schluff/i })).toHaveValue(25)
    expect(screen.getByRole('spinbutton', { name: /ton/i })).toHaveValue(10)
    expect(screen.getByText(/Sl3 – lehmiger Sand/)).toBeInTheDocument()
  })

  it('falls back to neutral fractions for conditions without a region', () => {
    renderDialog(SoilCondition.Unknown)
    expect(screen.getByRole('spinbutton', { name: /sand/i })).toHaveValue(33)
    expect(screen.getByRole('spinbutton', { name: /schluff/i })).toHaveValue(34)
    expect(screen.getByRole('spinbutton', { name: /ton/i })).toHaveValue(33)
  })

  it('rebalances the other fields and updates the result on change', () => {
    renderDialog(SoilCondition.Sl3)
    fireEvent.change(screen.getByRole('spinbutton', { name: /ton/i }), { target: { value: '80' } })
    expect(screen.getByRole('spinbutton', { name: /sand/i })).toHaveValue(14)
    expect(screen.getByRole('spinbutton', { name: /schluff/i })).toHaveValue(6)
    expect(screen.getByText(/Tt – reiner Ton/)).toBeInTheDocument()
  })

  it('applies the determined condition and closes', async () => {
    renderDialog(SoilCondition.Sl3)
    fireEvent.change(screen.getByRole('spinbutton', { name: /ton/i }), { target: { value: '80' } })
    await userEvent.click(screen.getByRole('button', { name: /übernehmen/i }))
    expect(mockOnApply).toHaveBeenCalledWith(SoilCondition.Tt)
    expect(mockOnOpenChange).toHaveBeenCalledWith(false)
  })

  it('does not apply when canceled', async () => {
    renderDialog(SoilCondition.Sl3)
    await userEvent.click(screen.getByRole('button', { name: /abbrechen/i }))
    expect(mockOnApply).not.toHaveBeenCalled()
    expect(mockOnOpenChange).toHaveBeenCalledWith(false)
  })
})
