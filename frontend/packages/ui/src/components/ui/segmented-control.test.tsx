import { describe, it, expect, vi, afterEach } from 'vitest'
import { render, screen, cleanup } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { SegmentedControl } from './segmented-control'

const options = [
  { value: 'none', label: 'Kein' },
  { value: 'view', label: 'Ansehen' },
  { value: 'edit', label: 'Bearbeiten' },
  { value: 'manage', label: 'Verwalten' },
]

afterEach(cleanup)

describe('SegmentedControl', () => {
  it('renders a labelled radiogroup', () => {
    render(
      <SegmentedControl
        options={options}
        value="edit"
        onChange={() => {}}
        ariaLabel="Zugriffsstufe für Bäume"
      />,
    )
    expect(screen.getByRole('radiogroup', { name: 'Zugriffsstufe für Bäume' })).toBeInTheDocument()
    expect(screen.getByRole('radio', { name: 'Bearbeiten' })).toBeChecked()
    expect(screen.getByRole('radio', { name: 'Kein' })).not.toBeChecked()
  })

  it('leaves every option unchecked when the value is null', () => {
    render(
      <SegmentedControl options={options} value={null} onChange={() => {}} ariaLabel="Stufe" />,
    )
    for (const radio of screen.getAllByRole('radio')) {
      expect(radio).not.toBeChecked()
    }
  })

  it('reports the clicked option', async () => {
    const onChange = vi.fn()
    render(
      <SegmentedControl options={options} value="none" onChange={onChange} ariaLabel="Stufe" />,
    )
    await userEvent.click(screen.getByRole('radio', { name: 'Verwalten' }))
    expect(onChange).toHaveBeenCalledWith('manage')
  })

  it('moves the selection with the arrow keys', async () => {
    const onChange = vi.fn()
    render(
      <SegmentedControl options={options} value="view" onChange={onChange} ariaLabel="Stufe" />,
    )
    screen.getByRole('radio', { name: 'Ansehen' }).focus()
    await userEvent.keyboard('{ArrowRight}')
    expect(onChange).toHaveBeenCalledWith('edit')
  })

  it('wraps around at the end', async () => {
    const onChange = vi.fn()
    render(
      <SegmentedControl options={options} value="manage" onChange={onChange} ariaLabel="Stufe" />,
    )
    screen.getByRole('radio', { name: 'Verwalten' }).focus()
    await userEvent.keyboard('{ArrowRight}')
    expect(onChange).toHaveBeenCalledWith('none')
  })

  it('reports nothing while disabled', async () => {
    const onChange = vi.fn()
    render(
      <SegmentedControl
        options={options}
        value="view"
        onChange={onChange}
        ariaLabel="Stufe"
        disabled
      />,
    )
    await userEvent.click(screen.getByRole('radio', { name: 'Verwalten' }))
    expect(onChange).not.toHaveBeenCalled()
  })

  it('renders buttons with type="button" so they never submit a form', () => {
    render(<SegmentedControl options={options} value="view" onChange={() => {}} ariaLabel="Stufe" />)
    for (const radio of screen.getAllByRole('radio')) {
      expect(radio).toHaveAttribute('type', 'button')
    }
  })
})
