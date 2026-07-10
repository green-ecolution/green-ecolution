import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { act, cleanup, render, screen, userEvent } from '@/test/utils'
import type { Map as MaplibreMap } from 'maplibre-gl'
import { toast } from '@green-ecolution/ui'
import { createFakeMap } from '@/test/mocks/fakeMaplibreMap'
import { MapContext } from './MapContext'
import GpsButton from './GpsButton'

afterEach(cleanup)

const watchPosition = vi.fn()
const clearWatch = vi.fn()

beforeEach(() => {
  vi.clearAllMocks()
  Object.defineProperty(globalThis.navigator, 'geolocation', {
    value: { watchPosition, clearWatch },
    configurable: true,
  })
  watchPosition.mockReturnValue(42)
})

const position = (lng: number, lat: number, accuracy = 25) =>
  ({ coords: { longitude: lng, latitude: lat, accuracy } }) as GeolocationPosition

const renderGps = () => {
  const map = createFakeMap()
  const view = render(
    <MapContext value={map as unknown as MaplibreMap}>
      <GpsButton />
    </MapContext>,
  )
  return { map, ...view }
}

describe('GpsButton', () => {
  it('starts a geolocation watch, draws the position and centers once', async () => {
    const { map } = renderGps()
    await userEvent.click(screen.getByRole('button', { name: 'Eigenen Standort anzeigen' }))
    expect(watchPosition).toHaveBeenCalledOnce()

    const onPosition = watchPosition.mock.calls[0][0] as (p: GeolocationPosition) => void
    act(() => onPosition(position(9.43, 54.79)))

    expect(map.getSource('gps-dot')?.setData).toHaveBeenCalled()
    expect(map.getSource('gps-accuracy')?.setData).toHaveBeenCalled()
    expect(map.easeTo).toHaveBeenCalledWith({ center: [9.43, 54.79], zoom: 16 })

    act(() => onPosition(position(9.44, 54.8)))
    expect(map.easeTo).toHaveBeenCalledOnce()
  })

  it('stops and hints when the position is outside the map bounds', async () => {
    const infoSpy = vi.spyOn(toast, 'info').mockImplementation(() => '')
    const { map } = renderGps()
    map.setBoundsContains(false)
    await userEvent.click(screen.getByRole('button', { name: 'Eigenen Standort anzeigen' }))

    const onPosition = watchPosition.mock.calls[0][0] as (p: GeolocationPosition) => void
    act(() => onPosition(position(13.4, 52.5)))

    expect(infoSpy).toHaveBeenCalledWith('Position außerhalb des Kartenbereichs')
    expect(clearWatch).toHaveBeenCalledWith(42)
    expect(screen.getByRole('button')).toHaveAttribute('aria-pressed', 'false')
  })

  it('deactivates with a toast when geolocation errors', async () => {
    const errorSpy = vi.spyOn(toast, 'error').mockImplementation(() => '')
    renderGps()
    await userEvent.click(screen.getByRole('button', { name: 'Eigenen Standort anzeigen' }))

    const onError = watchPosition.mock.calls[0][1] as (e: GeolocationPositionError) => void
    act(() => onError({ code: 1, PERMISSION_DENIED: 1 } as GeolocationPositionError))

    expect(errorSpy).toHaveBeenCalledWith('Standortzugriff verweigert')
    expect(clearWatch).toHaveBeenCalledWith(42)
    expect(screen.getByRole('button')).toHaveAttribute('aria-pressed', 'false')
  })

  it('cleans up watch and layers on unmount', async () => {
    const { map, unmount } = renderGps()
    await userEvent.click(screen.getByRole('button', { name: 'Eigenen Standort anzeigen' }))
    const onPosition = watchPosition.mock.calls[0][0] as (p: GeolocationPosition) => void
    act(() => onPosition(position(9.43, 54.79)))

    unmount()
    expect(clearWatch).toHaveBeenCalledWith(42)
    expect(map.removeSource).toHaveBeenCalledWith('gps-dot')
    expect(map.removeSource).toHaveBeenCalledWith('gps-accuracy')
  })
})
