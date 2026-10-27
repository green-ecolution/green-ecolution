import { describe, it, expect, afterEach } from 'vitest'
import { cleanup, render, screen, userEvent } from '@/test/utils'
import MapStatusLegend from './MapStatusLegend'

afterEach(cleanup)

describe('MapStatusLegend', () => {
  it('is collapsed by default', () => {
    render(<MapStatusLegend />)
    expect(screen.getByRole('button', { name: 'Legende anzeigen' })).toBeInTheDocument()
    expect(screen.queryByText('Bewässerungsstatus')).not.toBeInTheDocument()
  })

  it('shows all five watering statuses when expanded', async () => {
    render(<MapStatusLegend />)
    await userEvent.click(screen.getByRole('button', { name: 'Legende anzeigen' }))
    for (const label of [
      'Sehr trocken',
      'Leicht trocken',
      'In Ordnung',
      'Soeben bewässert',
      'Unbekannt',
    ]) {
      expect(screen.getByText(label)).toBeInTheDocument()
    }
  })

  it('collapses again via the toggle button', async () => {
    render(<MapStatusLegend />)
    await userEvent.click(screen.getByRole('button', { name: 'Legende anzeigen' }))
    await userEvent.click(screen.getByRole('button', { name: 'Legende ausblenden' }))
    expect(screen.queryByText('Bewässerungsstatus')).not.toBeInTheDocument()
  })

  it('collapses when clicking outside', async () => {
    render(
      <div>
        <MapStatusLegend />
        <button>Außerhalb</button>
      </div>,
    )
    await userEvent.click(screen.getByRole('button', { name: 'Legende anzeigen' }))
    await userEvent.click(screen.getByRole('button', { name: 'Außerhalb' }))
    expect(screen.queryByText('Bewässerungsstatus')).not.toBeInTheDocument()
  })
})
