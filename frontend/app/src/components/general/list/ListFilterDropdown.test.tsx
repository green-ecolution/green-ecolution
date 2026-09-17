import { describe, it, expect, vi } from 'vitest'
import { render, screen, fireEvent } from '@testing-library/react'
import ListFilterDropdown from './ListFilterDropdown'

const options = [
  { value: 'critical', label: 'Kritisch' },
  { value: 'unknown', label: 'Unbekannt' },
]

const openDropdown = (label: string) => {
  fireEvent.click(screen.getByRole('button', { name: new RegExp(label) }))
}

describe('ListFilterDropdown', () => {
  it('adds an unselected option to the current selection', () => {
    const onChange = vi.fn()
    render(
      <ListFilterDropdown
        label="Zustand"
        options={options}
        value={['unknown']}
        onChange={onChange}
        emptyText="Keine Treffer"
      />,
    )

    openDropdown('Zustand')
    fireEvent.click(screen.getByRole('option', { name: 'Kritisch' }))

    expect(onChange).toHaveBeenCalledExactlyOnceWith(['unknown', 'critical'])
  })

  it('removes an already-selected option, leaving the others', () => {
    const onChange = vi.fn()
    render(
      <ListFilterDropdown
        label="Zustand"
        options={options}
        value={['critical', 'unknown']}
        onChange={onChange}
        emptyText="Keine Treffer"
      />,
    )

    openDropdown('Zustand')
    fireEvent.click(screen.getByRole('option', { name: 'Kritisch' }))

    expect(onChange).toHaveBeenCalledExactlyOnceWith(['unknown'])
  })

  it('shows the count badge only once a selection exists', () => {
    const { rerender } = render(
      <ListFilterDropdown
        label="Zustand"
        options={options}
        value={[]}
        onChange={vi.fn()}
        emptyText="Keine Treffer"
      />,
    )

    expect(screen.queryByText('1')).not.toBeInTheDocument()

    rerender(
      <ListFilterDropdown
        label="Zustand"
        options={options}
        value={['critical']}
        onChange={vi.fn()}
        emptyText="Keine Treffer"
      />,
    )

    expect(screen.getByText('1')).toBeInTheDocument()
  })

  it('exposes the selected option as checked to assistive technology', () => {
    render(
      <ListFilterDropdown
        label="Zustand"
        options={options}
        value={['critical']}
        onChange={vi.fn()}
        emptyText="Keine Treffer"
      />,
    )

    openDropdown('Zustand')

    expect(screen.getByRole('option', { name: 'Kritisch' })).toHaveAttribute('aria-checked', 'true')
    expect(screen.getByRole('option', { name: 'Unbekannt' })).toHaveAttribute('aria-checked', 'false')
  })
})
