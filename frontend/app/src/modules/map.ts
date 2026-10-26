import { z } from 'zod'
import { MAP_ZOOM_SCALE_MAX, MAP_ZOOM_SCALE_MIN } from '@/lib/mapConfig'
import type { ModuleManifest } from './types'

export const coreMapSearchSchema = z.object({
  selected: z.string().optional(),
  // No default: the absence of a deep link is what lets the organization's own
  // centre decide where the map opens. A hardcoded fallback here would win over
  // it on every plain /map visit.
  lat: z.number().optional().catch(undefined),
  lng: z.number().optional().catch(undefined),
  sensorId: z.string().optional(),
  // Checked against the scale, not against a working range: how far this
  // organization lets its map zoom is a setting, and MapLibre clamps to it.
  zoom: z
    .number()
    .int()
    .min(MAP_ZOOM_SCALE_MIN)
    .max(MAP_ZOOM_SCALE_MAX)
    .optional()
    .catch(undefined),
})

/** Module keys stay loose here; each module reads its own keys through its own schema. */
export type MapSearch = z.output<typeof coreMapSearchSchema> & Record<string, unknown>

export function mapSearchSchemaFor(manifests: readonly ModuleManifest[]): z.ZodObject {
  return manifests.reduce<z.ZodObject>(
    (schema, { map }) => (map ? schema.extend(map.searchSchema.shape) : schema),
    coreMapSearchSchema,
  )
}

export function interactiveLayerIdsFor(manifests: readonly ModuleManifest[]): string[] {
  return manifests.flatMap(({ map }) => map?.interactiveLayerIds ?? [])
}
