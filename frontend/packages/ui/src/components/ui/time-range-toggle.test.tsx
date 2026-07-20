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
  it('marks the active option with aria-pressed', () => {
    render(<TimeRangeToggle options={options} value="7d" onChange={() => {}} />)
    expect(screen.getByRole('button', { name: '7 Tage' })).toHaveAttribute('aria-pressed', 'true')
    expect(screen.getByRole('button', { name: '24 h' })).toHaveAttribute('aria-pressed', 'false')
  })

  it('reports the clicked option', async () => {
    const onChange = vi.fn()
    render(<TimeRangeToggle options={options} value="7d" onChange={onChange} />)
    await userEvent.click(screen.getByRole('button', { name: '24 h' }))
    expect(onChange).toHaveBeenCalledWith('24h')
  })
})
