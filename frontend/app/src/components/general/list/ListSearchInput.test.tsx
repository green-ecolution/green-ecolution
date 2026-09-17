import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { render, screen, fireEvent, act } from '@testing-library/react'
import ListSearchInput from './ListSearchInput'

describe('ListSearchInput', () => {
  beforeEach(() => vi.useFakeTimers())
  afterEach(() => vi.useRealTimers())

  it('reports the trimmed value only after the debounce elapses', () => {
    const onChange = vi.fn()
    render(<ListSearchInput value="" onChange={onChange} label="Bäume suchen" debounceMs={300} />)

    fireEvent.change(screen.getByLabelText('Bäume suchen'), { target: { value: ' Quercus ' } })
    expect(onChange).not.toHaveBeenCalled()

    act(() => {
      vi.advanceTimersByTime(300)
    })
    expect(onChange).toHaveBeenCalledExactlyOnceWith('Quercus')
  })

  it('does not fire again when the debounced value is unchanged', () => {
    const onChange = vi.fn()
    render(<ListSearchInput value="Eiche" onChange={onChange} label="Bäume suchen" debounceMs={300} />)

    act(() => {
      vi.advanceTimersByTime(300)
    })
    expect(onChange).not.toHaveBeenCalled()
  })

  it('clears the field through the clear button', () => {
    const onChange = vi.fn()
    render(<ListSearchInput value="Eiche" onChange={onChange} label="Bäume suchen" debounceMs={300} />)

    fireEvent.click(screen.getByRole('button', { name: /leeren/i }))
    act(() => {
      vi.advanceTimersByTime(300)
    })
    expect(onChange).toHaveBeenCalledExactlyOnceWith('')
  })

  it('adopts a value changed externally, e.g. by a filter reset', () => {
    const onChange = vi.fn()
    const { rerender } = render(
      <ListSearchInput value="Eiche" onChange={onChange} label="Bäume suchen" debounceMs={300} />,
    )

    rerender(<ListSearchInput value="" onChange={onChange} label="Bäume suchen" debounceMs={300} />)

    expect(screen.getByLabelText('Bäume suchen')).toHaveValue('')

    act(() => {
      vi.advanceTimersByTime(300)
    })
    expect(onChange).not.toHaveBeenCalled()
  })
})
