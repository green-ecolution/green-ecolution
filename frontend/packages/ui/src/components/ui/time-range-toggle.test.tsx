import { describe, it, expect, vi, afterEach } from 'vitest'
import { render, screen, cleanup } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { TimeRangeToggle } from './time-range-toggle'

const options = [
  { value: '24h', label: '24 h' },
  { value: '7d', label: '7 Tage' },
]

afterEach(cleanup)

describe('TimeRangeToggle', () => {
  it('marks the active option as the checked radio', () => {
    render(<TimeRangeToggle options={options} value="7d" onChange={() => {}} />)
    expect(screen.getByRole('radio', { name: '7 Tage' })).toBeChecked()
    expect(screen.getByRole('radio', { name: '24 h' })).not.toBeChecked()
  })

  it('reports the clicked option', async () => {
    const onChange = vi.fn()
    render(<TimeRangeToggle options={options} value="7d" onChange={onChange} />)
    await userEvent.click(screen.getByRole('radio', { name: '24 h' }))
    expect(onChange).toHaveBeenCalledWith('24h')
  })

  it('renders buttons with type="button" so they never trigger a form submit', () => {
    render(<TimeRangeToggle options={options} value="7d" onChange={() => {}} />)
    for (const radio of screen.getAllByRole('radio')) {
      expect(radio).toHaveAttribute('type', 'button')
    }
  })
})
