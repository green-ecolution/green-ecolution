import { describe, it, expect, afterEach } from 'vitest'
import { act, cleanup, render, screen, userEvent } from '@/test/utils'
import type { Map as MaplibreMap } from 'maplibre-gl'
import { createFakeMap } from '@/test/mocks/fakeMaplibreMap'
import { MapContext } from './MapContext'
import Map3DToggle from './Map3DToggle'

afterEach(cleanup)

const render3D = () => {
  const map = createFakeMap()
  render(
    <MapContext value={map as unknown as MaplibreMap}>
      <Map3DToggle />
    </MapContext>,
  )
  return map
}

describe('Map3DToggle', () => {
  it('tilts the camera to 60 degrees when enabling 3D', async () => {
    const map = render3D()
    await userEvent.click(screen.getByRole('button', { name: 'Zur 3D-Ansicht wechseln' }))
    expect(map.easeTo).toHaveBeenCalledWith({ pitch: 60 })
  })

  it('derives its active state from the actual map pitch', async () => {
    const map = render3D()
    expect(screen.getByRole('button')).toHaveAttribute('aria-pressed', 'false')
    act(() => map.setPitch(60))
    expect(screen.getByRole('button')).toHaveAttribute('aria-pressed', 'true')
    await userEvent.click(screen.getByRole('button', { name: 'Zur 2D-Ansicht wechseln' }))
    expect(map.easeTo).toHaveBeenCalledWith({ pitch: 0 })
  })
})
