import { useCallback, useState } from 'react'
import { translateIssue, validateMapView, type ValidationIssue } from '@green-ecolution/domain-wasm'
import {
  SettingOriginDto,
  type MapViewDto,
  type OrganizationSettingsResponse,
  type OrganizationSettingsUpdateRequest,
} from '@/api/backendApi'
import { useIssueTranslator } from '@/lib/i18n/validation'

export interface MapViewFieldDraft {
  own: boolean
  /** Always a whole viewport: centre and box only mean anything together. */
  value: MapViewDto
}

export interface SettingsDraft {
  mapView: MapViewFieldDraft
  descendantsMayOverride: boolean
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
  mapView: {
    own: response.mapView.origin === SettingOriginDto.Own,
    value: response.mapView.value,
  },
  descendantsMayOverride: response.descendantsMayOverride,
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

  const setMapViewOwn = (own: boolean) => {
    setDraft((current) => {
      if (!current || !server) return current
      if (!own) return { ...current, mapView: { ...current.mapView, own: false } }
      // Take over what is currently in force, so the picker opens on the view
      // the person already sees rather than somewhere they have to find again.
      return { ...current, mapView: { own: true, value: server.mapView.value } }
    })
  }

  const setMapView = (value: MapViewDto) => {
    setDraft((current) => (current ? { ...current, mapView: { own: true, value } } : current))
  }

  // Dropping the limit keeps the centre: the map still has to open somewhere,
  // and restoring one should not send the person hunting for their own town.
  // Box and zoom range go together, so both leave and both come back.
  const setMapViewRestricted = (restricted: boolean) => {
    setDraft((current) => {
      if (!current) return current
      const { center, bbox, minZoom, maxZoom } = current.mapView.value
      if (!restricted) {
        return {
          ...current,
          mapView: {
            own: true,
            value: { center, bbox: null, minZoom: undefined, maxZoom: undefined },
          },
        }
      }
      // Coming back from unrestricted there is nothing to restore, so start
      // from limits around the centre that the picker can then adjust.
      const limits = bbox ? { bbox, minZoom, maxZoom } : limitsAround(center)
      return { ...current, mapView: { own: true, value: { center, ...limits } } }
    })
  }

  const setMapViewZoom = (which: 'minZoom' | 'maxZoom', level: number) => {
    setDraft((current) =>
      current
        ? {
            ...current,
            mapView: {
              own: true,
              value: { ...current.mapView.value, [which]: level },
            },
          }
        : current,
    )
  }

  const setDescendantsMayOverride = (value: boolean) => {
    setDraft((current) => (current ? { ...current, descendantsMayOverride: value } : current))
  }

  const mapViewIssue = draft?.mapView.own ? checkMapView(draft.mapView.value) : null

  const errors = {
    mapView: mapViewIssue ? translateIssue(mapViewIssue, translate) : null,
  }

  const valid = errors.mapView === null

  const toRequest = (): OrganizationSettingsUpdateRequest | null => {
    // A non-numeric own value would otherwise serialize to `null` over JSON
    // and silently read as "give up the own value" on the server. Refuse to
    // build a body at all while invalid; the caller already sees the error
    // via `errors`, so nothing is lost by withholding the request.
    if (!draft || !server || !valid) return null
    const body: OrganizationSettingsUpdateRequest = {}

    const mapViewWasOwn = server.mapView.origin === SettingOriginDto.Own
    if (!draft.mapView.own) {
      if (mapViewWasOwn) body.mapView = null
    } else if (!mapViewWasOwn || !sameMapView(draft.mapView.value, server.mapView.value)) {
      body.mapView = draft.mapView.value
    }

    if (draft.descendantsMayOverride !== server.descendantsMayOverride) {
      body.descendantsMayOverride = draft.descendantsMayOverride
    }

    return Object.keys(body).length > 0 ? body : null
  }

  const dirty = toRequest() !== null

  return {
    draft,
    dirty,
    errors,
    valid,
    load,
    setMapViewOwn,
    setMapViewRestricted,
    setMapViewZoom,
    setMapView,
    setDescendantsMayOverride,
    toRequest,
  }
}
