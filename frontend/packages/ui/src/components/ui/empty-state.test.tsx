import { describe, it, expect, afterEach } from 'vitest'
import { render, screen, cleanup, within } from '@testing-library/react'
import { EmptyState } from './empty-state'

afterEach(cleanup)

const benefits = {
  title: 'Wofür Unterorganisationen gut sind',
  items: [
    { icon: <svg />, title: 'Eigenes Team', text: 'Eigene Mitarbeitende.' },
    { icon: <svg />, title: 'Getrennte Daten', text: 'Nur eigene Daten.' },
    { icon: <svg />, title: 'Alles im Blick', text: 'Überblick bleibt.' },
  ],
}

describe('EmptyState', () => {
  it('shows the headline and the explanation', () => {
    render(<EmptyState title="Noch nichts da" description="Das ist ein Erklärsatz." />)
    expect(screen.getByRole('heading', { level: 3, name: 'Noch nichts da' })).toBeInTheDocument()
    expect(screen.getByText('Das ist ein Erklärsatz.')).toBeInTheDocument()
  })

  it('renders the action only when one is given', () => {
    const { rerender } = render(
      <EmptyState title="T" description="D" action={<button type="button">Anlegen</button>} />,
    )
    expect(screen.getByRole('button', { name: 'Anlegen' })).toBeInTheDocument()
    rerender(<EmptyState title="T" description="D" />)
    expect(screen.queryByRole('button')).not.toBeInTheDocument()
  })

  it('exposes the example as an image with its label and hides the content', () => {
    render(
      <EmptyState
        title="T"
        description="D"
        example={{
          caption: 'So könnte es aussehen',
          label: 'Beispiel: TBZ mit Betriebshof',
          content: <span>Betriebshof</span>,
        }}
      />,
    )
    const example = screen.getByRole('img', { name: 'Beispiel: TBZ mit Betriebshof' })
    expect(screen.getByText('So könnte es aussehen')).toBeInTheDocument()
    expect(within(example).getByText('Betriebshof').closest('[aria-hidden="true"]')).not.toBeNull()
  })

  it('lists the benefits under their heading', () => {
    render(<EmptyState title="T" description="D" benefits={benefits} />)
    expect(
      screen.getByRole('heading', { level: 4, name: 'Wofür Unterorganisationen gut sind' }),
    ).toBeInTheDocument()
    const list = screen.getByRole('list')
    expect(within(list).getAllByRole('listitem')).toHaveLength(3)
    expect(within(list).getByText('Getrennte Daten')).toBeInTheDocument()
  })

  it('omits benefits and note when they are not given', () => {
    render(<EmptyState title="T" description="D" />)
    expect(screen.queryByRole('list')).not.toBeInTheDocument()
    expect(screen.queryByRole('heading', { level: 4 })).not.toBeInTheDocument()
    expect(screen.queryByText('Gut zu wissen')).not.toBeInTheDocument()
  })

  it('renders the note', () => {
    render(<EmptyState title="T" description="D" note="Gut zu wissen: Hinweis." />)
    expect(screen.getByText('Gut zu wissen: Hinweis.')).toBeInTheDocument()
  })
})
