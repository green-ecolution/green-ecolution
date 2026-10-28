import { describe, expect, it } from 'vitest'
import { render, screen } from '@testing-library/react'
import FillStateBadge from './FillStateBadge'

describe('FillStateBadge', () => {
  it('shows a full bin with its percentage', () => {
    render(<FillStateBadge state="full" percent={85} />)
    expect(screen.getByText('Voll · 85 %')).toBeInTheDocument()
  })

  it('shows a normal bin with its percentage', () => {
    render(<FillStateBadge state="normal" percent={40} />)
    expect(screen.getByText('Normal · 40 %')).toBeInTheDocument()
  })

  it('leaves the percentage out for an emptied bin, since the reading predates the emptying', () => {
    render(<FillStateBadge state="empty" percent={85} />)
    expect(screen.getByText('Geleert')).toBeInTheDocument()
    expect(screen.queryByText(/85/)).not.toBeInTheDocument()
  })

  it('shows an unknown state without a percentage', () => {
    render(<FillStateBadge state="unknown" />)
    expect(screen.getByText('Unbekannt')).toBeInTheDocument()
  })

  it('shows the state alone when no percentage is known', () => {
    render(<FillStateBadge state="normal" />)
    expect(screen.getByText('Normal')).toBeInTheDocument()
  })

  it('colors the badge by state', () => {
    const { rerender } = render(<FillStateBadge state="full" percent={85} />)
    expect(screen.getByText('Voll · 85 %').closest('[data-fill-state]')).toHaveClass('text-red')
    rerender(<FillStateBadge state="normal" percent={40} />)
    expect(screen.getByText('Normal · 40 %').closest('[data-fill-state]')).toHaveClass(
      'text-green-dark',
    )
    rerender(<FillStateBadge state="empty" />)
    expect(screen.getByText('Geleert').closest('[data-fill-state]')).toHaveClass('text-dark-600')
    rerender(<FillStateBadge state="unknown" />)
    expect(screen.getByText('Unbekannt').closest('[data-fill-state]')).toHaveClass('border-dashed')
  })
})
