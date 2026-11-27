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

  it('exposes the section as a region named by the headline', () => {
    render(<EmptyState title="Noch nichts da" description="D" />)
    expect(screen.getByRole('region', { name: 'Noch nichts da' })).toBeInTheDocument()
  })

  it('renders the benefits zone only when benefits are given', () => {
    const { container, rerender } = render(
      <EmptyState title="T" description="D" benefits={benefits} />,
    )
    expect(container.querySelector('[data-slot="empty-state-benefits"]')).not.toBeNull()
    rerender(<EmptyState title="T" description="D" />)
    expect(container.querySelector('[data-slot="empty-state-benefits"]')).toBeNull()
    expect(screen.queryByRole('list')).not.toBeInTheDocument()
    expect(screen.queryByRole('heading', { level: 4 })).not.toBeInTheDocument()
  })

  it('renders the example zone only when an example is given', () => {
    const example = { caption: 'So könnte es aussehen', label: 'Beispiel', content: <span>x</span> }
    const { container, rerender } = render(
      <EmptyState title="T" description="D" example={example} />,
    )
    expect(container.querySelector('[data-slot="empty-state-example"]')).not.toBeNull()
    rerender(<EmptyState title="T" description="D" />)
    expect(container.querySelector('[data-slot="empty-state-example"]')).toBeNull()
    expect(screen.queryByRole('img')).not.toBeInTheDocument()
    expect(screen.queryByText('So könnte es aussehen')).not.toBeInTheDocument()
  })

  it('renders the note zone only when a note is given', () => {
    const { container, rerender } = render(
      <EmptyState title="T" description="D" note="Gut zu wissen: Hinweis." />,
    )
    expect(container.querySelector('[data-slot="empty-state-note"]')).not.toBeNull()
    rerender(<EmptyState title="T" description="D" />)
    expect(container.querySelector('[data-slot="empty-state-note"]')).toBeNull()
  })

  it('renders the note', () => {
    render(<EmptyState title="T" description="D" note="Gut zu wissen: Hinweis." />)
    expect(screen.getByText('Gut zu wissen: Hinweis.')).toBeInTheDocument()
  })
})
