import { describe, it, expect } from 'vitest'
import { act } from 'react'
import { renderHook } from '@testing-library/react'
import type { OrganizationSettingsResponse } from '@/api/backendApi'
import { useOrganizationSettingsDraft } from './useOrganizationSettingsDraft'

const MAP_VIEW = {
  center: [54.7923, 9.4358],
  bbox: [54.7148, 9.2858, 54.8601, 9.5838],
  minZoom: 13,
  maxZoom: 18,
}

const response = {
  mapView: { value: MAP_VIEW, origin: 'default', lastChange: null },
  descendantsMayOverride: true,
  enforcedBy: null,
} as unknown as OrganizationSettingsResponse

describe('useOrganizationSettingsDraft', () => {
  it('starts clean and sends nothing', () => {
    const { result } = renderHook(() => useOrganizationSettingsDraft())
    act(() => result.current.load(response))
    expect(result.current.dirty).toBe(false)
    expect(result.current.toRequest()).toBeNull()
  })

  it('takes over the viewport in force when the map view is switched to own', () => {
    const { result } = renderHook(() => useOrganizationSettingsDraft())
    act(() => result.current.load(response))
    act(() => result.current.setMapViewOwn(true))
    expect(result.current.draft?.mapView.value).toEqual(MAP_VIEW)
    expect(result.current.toRequest()).toEqual({ mapView: MAP_VIEW })
  })

  it('sends the moved viewport as a whole', () => {
    const moved = {
      center: [53.55, 9.99],
      bbox: [53.4, 9.75, 53.7, 10.25],
      minZoom: 13,
      maxZoom: 18,
    }
    const { result } = renderHook(() => useOrganizationSettingsDraft())
    act(() => result.current.load(response))
    act(() => result.current.setMapView(moved))
    expect(result.current.toRequest()).toEqual({ mapView: moved })
  })

  it('sends null when an own viewport is given back to inheritance', () => {
    const owned = {
      ...response,
      mapView: { value: MAP_VIEW, origin: 'own', ownValue: MAP_VIEW, lastChange: null },
    } as unknown as OrganizationSettingsResponse
    const { result } = renderHook(() => useOrganizationSettingsDraft())
    act(() => result.current.load(owned))
    act(() => result.current.setMapViewOwn(false))
    expect(result.current.toRequest()).toEqual({ mapView: null })
  })

  it('drops the box but keeps the centre when the limit is switched off', () => {
    const { result } = renderHook(() => useOrganizationSettingsDraft())
    act(() => result.current.load(response))
    act(() => result.current.setMapViewRestricted(false))

    expect(result.current.draft?.mapView.value.bbox).toBeNull()
    expect(result.current.draft?.mapView.value.center).toEqual(MAP_VIEW.center)
    expect(result.current.toRequest()).toEqual({
      mapView: { center: MAP_VIEW.center, bbox: null, minZoom: undefined, maxZoom: undefined },
    })
  })

  it('drops the zoom range along with the box', () => {
    const { result } = renderHook(() => useOrganizationSettingsDraft())
    act(() => result.current.load(response))
    act(() => result.current.setMapViewRestricted(false))

    expect(result.current.draft?.mapView.value.minZoom).toBeUndefined()
    expect(result.current.draft?.mapView.value.maxZoom).toBeUndefined()
  })

  it('sends a changed zoom level with the rest of the viewport', () => {
    const { result } = renderHook(() => useOrganizationSettingsDraft())
    act(() => result.current.load(response))
    act(() => result.current.setMapViewZoom('maxZoom', 20))

    expect(result.current.toRequest()).toEqual({
      mapView: { ...MAP_VIEW, maxZoom: 20 },
    })
  })

  it('refuses a zoom range that runs backwards', () => {
    const { result } = renderHook(() => useOrganizationSettingsDraft())
    act(() => result.current.load(response))
    act(() => result.current.setMapViewZoom('minZoom', 20))

    expect(result.current.errors.mapView).not.toBeNull()
    expect(result.current.valid).toBe(false)
    expect(result.current.toRequest()).toBeNull()
  })

  // Nothing was stored to return to, so the centre has to supply the frame.
  it('offers a box around the centre when the limit is switched back on', () => {
    const { result } = renderHook(() => useOrganizationSettingsDraft())
    act(() => result.current.load(response))
    act(() => result.current.setMapViewRestricted(false))
    act(() => result.current.setMapViewRestricted(true))

    const bbox = result.current.draft?.mapView.value.bbox
    expect(bbox).not.toBeNull()
    const [swLat, swLng, neLat, neLng] = bbox!
    const [lat, lng] = MAP_VIEW.center
    expect(swLat).toBeLessThan(lat)
    expect(neLat).toBeGreaterThan(lat)
    expect(swLng).toBeLessThan(lng)
    expect(neLng).toBeGreaterThan(lng)
    expect(result.current.errors.mapView).toBeNull()
  })

  it('refuses a viewport whose centre lies outside its box', () => {
    const { result } = renderHook(() => useOrganizationSettingsDraft())
    act(() => result.current.load(response))
    act(() =>
      result.current.setMapView({
        center: [48.13, 11.58],
        bbox: [53.4, 9.75, 53.7, 10.25],
        minZoom: 13,
        maxZoom: 18,
      }),
    )
    expect(result.current.errors.mapView).not.toBeNull()
    expect(result.current.valid).toBe(false)
    expect(result.current.toRequest()).toBeNull()
  })
})
