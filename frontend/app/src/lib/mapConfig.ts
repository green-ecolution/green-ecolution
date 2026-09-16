// The pan limit and the zoom range are per-organization settings, resolved via
// `useMapView`; there are deliberately no constants for them here. What stays
// is the pre-resolution seed and the zoom a map opens at when its organization
// set no lower bound to open from.
export const MAP_DEFAULT_CENTER: [number, number] = [54.792277136221905, 9.43580607453268]
export const DEFAULT_OPEN_ZOOM = 13

/** The deepest level the tiles are cut to, matching the domain's `ZoomLevel`. */
export const MAP_ZOOM_SCALE_MIN = 0
export const MAP_ZOOM_SCALE_MAX = 24
