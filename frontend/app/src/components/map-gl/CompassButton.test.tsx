import { describe, it, expect, afterEach } from 'vitest'
import { act, cleanup, render, screen, userEvent } from '@/test/utils'
import type { Map as MaplibreMap } from 'maplibre-gl'
import { createFakeMap } from '@/test/mocks/fakeMaplibreMap'
import { MapContext } from './MapContext'
import CompassButton from './CompassButton'

afterEach(cleanup)

const renderCompass = () => {
  const map = createFakeMap()
  render(
    <MapContext value={map as unknown as MaplibreMap}>
      <CompassButton />
    </MapContext>,
  )
  return map
}

describe('CompassButton', () => {
  it('rotates the needle against the map bearing', () => {
    const map = renderCompass()
    act(() => map.setBearing(90))
    expect(screen.getByTestId('compass-needle')).toHaveStyle({ transform: 'rotate(-90deg)' })
  })

  it('resets bearing and pitch on click', async () => {
    const map = renderCompass()
    act(() => map.setBearing(45))
    await userEvent.click(screen.getByRole('button', { name: 'Karte nach Norden ausrichten' }))
    expect(map.easeTo).toHaveBeenCalledWith({ bearing: 0, pitch: 0 })
  })

  it('removes its map listeners on unmount', () => {
    const map = createFakeMap()
    const { unmount } = render(
      <MapContext value={map as unknown as MaplibreMap}>
        <CompassButton />
      </MapContext>,
    )
    unmount()
    expect(map.off).toHaveBeenCalledWith('rotate', expect.any(Function))
    expect(map.off).toHaveBeenCalledWith('pitch', expect.any(Function))
  })
})
