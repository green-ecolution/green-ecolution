import { useCallback, useState } from 'react'
import { translateIssue, validateMapView, type ValidationIssue } from '@green-ecolution/domain-wasm'
import {
  type MapViewDto,
  type OrganizationSettingsResponse,
  type OrganizationSettingsUpdateRequest,
} from '@/api/backendApi'
import { useIssueTranslator } from '@/lib/i18n/validation'

export interface SettingsDraft {
  mapView: MapViewDto
}

// wasm-bindgen emits `(input: number) => any`; tighten the return type at the boundary,
// same pattern as the draft resolvers in @green-ecolution/domain-wasm.
const mapViewValidator = validateMapView as (
  centerLat: number,
  centerLng: number,
  swLat?: number | null,
  swLng?: number | null,
  neLat?: number | null,
  neLng?: number | null,
  minZoom?: number | null,
  maxZoom?: number | null,
) => ValidationIssue | null

export const checkMapView = (view: MapViewDto): ValidationIssue | null =>
  mapViewValidator(
    view.center[0],
    view.center[1],
    view.bbox?.[0],
    view.bbox?.[1],
    view.bbox?.[2],
    view.bbox?.[3],
    view.minZoom,
    view.maxZoom,
  )

/** Whether this viewport holds the map to a working area at all. The box and
 *  the zoom range are one decision, so one of them answers for all three. */
export const isRestricted = (view: MapViewDto): boolean => view.bbox != null

const sameBbox = (a: MapViewDto['bbox'], b: MapViewDto['bbox']): boolean => {
  if (!a || !b) return !a && !b
  return a.every((v, i) => v === b[i])
}

const sameMapView = (a: MapViewDto, b: MapViewDto): boolean =>
  a.center.every((v, i) => v === b.center[i]) &&
  sameBbox(a.bbox, b.bbox) &&
  a.minZoom === b.minZoom &&
  a.maxZoom === b.maxZoom

/** The frame a limit starts from when it is switched back on and there is no
 *  earlier one to return to: roughly a municipality around the centre, at the
 *  zoom range the application shipped with. */
const limitsAround = ([lat, lng]: number[]): Pick<MapViewDto, 'bbox' | 'minZoom' | 'maxZoom'> => ({
  bbox: [lat - 0.08, lng - 0.15, lat + 0.08, lng + 0.15],
  minZoom: 13,
  maxZoom: 18,
})

const draftOf = (response: OrganizationSettingsResponse): SettingsDraft => ({
  mapView: response.mapView.value,
})

export const useOrganizationSettingsDraft = () => {
  const translate = useIssueTranslator()
  const [server, setServer] = useState<OrganizationSettingsResponse | null>(null)
  const [draft, setDraft] = useState<SettingsDraft | null>(null)

  // Stable so a page may load it into an effect that follows the loaded settings.
  const load = useCallback((response: OrganizationSettingsResponse) => {
    setServer(response)
    setDraft(draftOf(response))
  }, [])

  const setMapView = (mapView: MapViewDto) => {
    setDraft((current) => (current ? { ...current, mapView } : current))
  }

  // Dropping the limit keeps the centre: the map still has to open somewhere,
  // and restoring one should not send the person hunting for their own town.
  // Box and zoom range go together, so both leave and both come back.
  const setMapViewRestricted = (restricted: boolean) => {
    setDraft((current) => {
      if (!current) return current
      const { center, bbox, minZoom, maxZoom } = current.mapView
      if (!restricted) {
        return {
          ...current,
          mapView: { center, bbox: null, minZoom: undefined, maxZoom: undefined },
        }
      }
      // Coming back from unrestricted there is nothing to restore, so start
      // from limits around the centre that the picker can then adjust.
      const limits = bbox ? { bbox, minZoom, maxZoom } : limitsAround(center)
      return { ...current, mapView: { center, ...limits } }
    })
  }

  const setMapViewZoom = (which: 'minZoom' | 'maxZoom', level: number) => {
    setDraft((current) =>
      current ? { ...current, mapView: { ...current.mapView, [which]: level } } : current,
    )
  }

  const mapViewIssue = draft ? checkMapView(draft.mapView) : null
  const errors = { mapView: mapViewIssue ? translateIssue(mapViewIssue, translate) : null }
  const valid = errors.mapView === null

  const toRequest = (): OrganizationSettingsUpdateRequest | null => {
    if (!draft || !server || !valid) return null
    return sameMapView(draft.mapView, server.mapView.value) ? null : { mapView: draft.mapView }
  }

  const dirty = toRequest() !== null

  return {
    draft,
    dirty,
    errors,
    valid,
    load,
    setMapViewRestricted,
    setMapViewZoom,
    setMapView,
    toRequest,
  }
}
