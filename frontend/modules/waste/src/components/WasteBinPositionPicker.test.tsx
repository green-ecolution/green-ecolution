import type { ReactNode } from 'react'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { act, render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import type { MapClickLngLat } from '@/components/map-gl/useMapClick'

const mapClick: { current?: (pos: MapClickLngLat) => void } = {}

// MapLibre needs WebGL, which jsdom does not provide.
vi.mock('@/components/map-gl/MapPreview', () => ({
  default: ({ children, ariaLabel }: { children?: ReactNode; ariaLabel?: string }) => (
    <div data-testid="map-preview" aria-label={ariaLabel}>
      {children}
    </div>
  ),
}))
vi.mock('@/components/map-gl/useMapClick', () => ({
  useMapClick: (onClick: (pos: MapClickLngLat) => void) => {
    mapClick.current = onClick
  },
}))
vi.mock('@/components/map-gl/DraggableMarker', () => ({
  default: ({
    lat,
    lng,
    onDragEnd,
  }: {
    lat: number
    lng: number
    onDragEnd?: (pos: { lat: number; lng: number }) => void
  }) => (
    <button type="button" onClick={() => onDragEnd?.({ lat: 54.8, lng: 9.45 })}>
      {`marker ${lat} ${lng}`}
    </button>
  ),
}))

const { default: WasteBinPositionPicker } = await import('./WasteBinPositionPicker')

describe('WasteBinPositionPicker', () => {
  const onChange = vi.fn()

  beforeEach(() => {
    onChange.mockClear()
    mapClick.current = undefined
  })

  it('asks for a click while no position is set', () => {
    render(<WasteBinPositionPicker onChange={onChange} />)
    expect(screen.getByText(/Klicke auf die Karte/)).toBeInTheDocument()
    expect(screen.queryByRole('button', { name: /marker/ })).not.toBeInTheDocument()
  })

  it('sets the position with the first map click', () => {
    render(<WasteBinPositionPicker onChange={onChange} />)
    act(() => mapClick.current?.({ lat: 54.78, lng: 9.43 }))
    expect(onChange).toHaveBeenCalledWith({ lat: 54.78, lng: 9.43 })
  })

  it('ignores further map clicks once a position is set', () => {
    render(<WasteBinPositionPicker lat={54.78} lng={9.43} onChange={onChange} />)
    act(() => mapClick.current?.({ lat: 1, lng: 2 }))
    expect(onChange).not.toHaveBeenCalled()
  })

  it('moves the position by dragging the marker', async () => {
    const user = userEvent.setup()
    render(<WasteBinPositionPicker lat={54.78} lng={9.43} onChange={onChange} />)
    await user.click(screen.getByRole('button', { name: 'marker 54.78 9.43' }))
    expect(onChange).toHaveBeenCalledWith({ lat: 54.8, lng: 9.45 })
    expect(screen.getByText('54.78000, 9.43000')).toBeInTheDocument()
  })

  it('shows the error in place of the hint', () => {
    render(<WasteBinPositionPicker onChange={onChange} error="Bitte Standort wählen" />)
    expect(screen.getByRole('alert')).toHaveTextContent('Bitte Standort wählen')
  })
})
