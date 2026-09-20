import { describe, it, expect, vi } from 'vitest'
import { render, screen, fireEvent } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
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

  it('announces the selected option to assistive technology', () => {
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

    const selectedOption = screen.getByRole('option', { name: 'Kritisch' })
    const describedById = selectedOption.getAttribute('aria-describedby')
    expect(describedById).toBeTruthy()
    expect(document.getElementById(describedById!)).toHaveTextContent('Ausgewählt')

    const unselectedOption = screen.getByRole('option', { name: 'Unbekannt' })
    const unselectedDescribedById = unselectedOption.getAttribute('aria-describedby')
    expect(unselectedDescribedById).toBeTruthy()
    expect(document.getElementById(unselectedDescribedById!)).toHaveTextContent('Nicht ausgewählt')
  })

  it('replaces the previous value in single mode', async () => {
    const onChange = vi.fn()
    const user = userEvent.setup()
    render(
      <ListFilterDropdown
        label="Sensor"
        mode="single"
        options={[
          { value: 'true', label: 'Mit Sensor' },
          { value: 'false', label: 'Ohne Sensor' },
        ]}
        value={['true']}
        onChange={onChange}
        emptyText="Keine Optionen"
      />,
    )

    await user.click(screen.getByRole('button', { name: /Sensor/ }))
    await user.click(screen.getByRole('option', { name: 'Ohne Sensor' }))

    expect(onChange).toHaveBeenCalledWith(['false'])
  })

  it('clears the value when the selected option is picked again in single mode', async () => {
    const onChange = vi.fn()
    const user = userEvent.setup()
    render(
      <ListFilterDropdown
        label="Sensor"
        mode="single"
        options={[{ value: 'true', label: 'Mit Sensor' }]}
        value={['true']}
        onChange={onChange}
        emptyText="Keine Optionen"
      />,
    )

    await user.click(screen.getByRole('button', { name: /Sensor/ }))
    await user.click(screen.getByRole('option', { name: 'Mit Sensor' }))

    expect(onChange).toHaveBeenCalledWith([])
  })
})
