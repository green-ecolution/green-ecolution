import { describe, it, expect, vi } from 'vitest'
import { render, screen, fireEvent, act } from '@testing-library/react'
import SensorTreeSearchInput from './SensorTreeSearchInput'

describe('SensorTreeSearchInput', () => {
  it('debounces onChange by 300ms', () => {
    vi.useFakeTimers()
    const onChange = vi.fn()
    render(<SensorTreeSearchInput value="" onChange={onChange} />)
    const input = screen.getByRole('searchbox')

    fireEvent.change(input, { target: { value: 'E' } })
    fireEvent.change(input, { target: { value: 'Ei' } })
    fireEvent.change(input, { target: { value: 'Eic' } })

    expect(onChange).not.toHaveBeenCalled()
    act(() => {
      vi.advanceTimersByTime(310)
    })
    expect(onChange).toHaveBeenCalledTimes(1)
    expect(onChange).toHaveBeenCalledWith('Eic')
    vi.useRealTimers()
  })

  it('clears the input when the clear button is clicked', () => {
    vi.useFakeTimers()
    const onChange = vi.fn()
    render(<SensorTreeSearchInput value="Eiche" onChange={onChange} />)

    fireEvent.click(screen.getByRole('button', { name: /Leeren/i }))
    void act(() => vi.advanceTimersByTime(310))

    expect(onChange).toHaveBeenLastCalledWith('')
    vi.useRealTimers()
  })

  it('hides the clear button when input is empty', () => {
    render(<SensorTreeSearchInput value="" onChange={vi.fn()} />)
    expect(screen.queryByRole('button', { name: /Leeren/i })).not.toBeInTheDocument()
  })

  it('syncs from external value changes', () => {
    const { rerender } = render(<SensorTreeSearchInput value="" onChange={vi.fn()} />)
    rerender(<SensorTreeSearchInput value="Buche" onChange={vi.fn()} />)
    expect(screen.getByRole('searchbox')).toHaveValue('Buche')
  })
})
