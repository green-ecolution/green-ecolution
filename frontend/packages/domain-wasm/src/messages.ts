import type { IssueTranslator, TranslatableIssue } from './types'

/**
 * Every key the core's Rust validators can emit, in catalog order. A module
 * lists the keys of its own validators itself.
 *
 * The texts live in the consuming app's `validation` namespace; this package
 * holds keys only. A test in the app checks the catalog against this list, so
 * a new Rust variant cannot arrive without a translation.
 */
export const VALIDATION_KEYS = [
  // Coordinate
  'coordinate.latitude.outOfRange',
  'coordinate.longitude.outOfRange',
  'coordinate.latitude.invalidFormat',
  'coordinate.longitude.invalidFormat',

  // Vehicle
  'vehicle.number_plate.empty',
  'vehicle.number_plate.tooShort',
  'vehicle.number_plate.tooLong',
  'vehicle.model.empty',
  'vehicle.model.tooShort',
  'vehicle.model.tooLong',
  'water_capacity.outOfRange',
  'vehicle.water_capacity.invalidFormat',
  'vehicle.cargo_volume_capacity.invalidFormat',
  'vehicle.capacities.empty',
  'vehicle.dimension.height.outOfRange',
  'vehicle.dimension.width.outOfRange',
  'vehicle.dimension.length.outOfRange',
  'vehicle.dimension.weight.outOfRange',
  'vehicle.dimension.height.invalidFormat',
  'vehicle.dimension.width.invalidFormat',
  'vehicle.dimension.length.invalidFormat',
  'vehicle.dimension.weight.invalidFormat',
  'vehicle.type.invalidFormat',
  'vehicle.driving_license.invalidFormat',
  'vehicle.status.invalidFormat',

  // Plan
  'plan.stop_ids.empty',
  'plan.driver_ids.empty',
  'plan.transporter_id.empty',
  'plan.transporter_id.outOfRange',
  'plan.trailer_id.outOfRange',
  'plan.driver_ids.invalidFormat',
  'plan.status.invalidFormat',
  'plan.date.outOfRange',
  'plan.start_point_name.empty',

  // Settings
  'settings.sensor_offline_after.outOfRange',
  'settings.defect_streak.outOfRange',
  'settings.defect_streak.invalidFormat',
  'settings.map_view.invalidFormat',

  // Bounding box, reachable through the map viewport setting.
  'geo.bbox.sw_lat.outOfRange',
  'geo.bbox.sw_lng.outOfRange',
  'geo.bbox.ne_lat.outOfRange',
  'geo.bbox.ne_lng.outOfRange',
  'geo.bbox.invalidFormat',

  // Provenance. Only reachable through the server's validation block, not the
  // in-browser validators: a provider is set by an import, never typed in.
  'provenance.provider.empty',
  'provenance.provider.tooLong',

  // User
  'user.email.empty',
  'user.email.invalidFormat',
  'user.username.empty',
  'user.username.tooLong',
  // Names are only checked by the server.
  'user.first_name.empty',
  'user.first_name.tooLong',
  'user.last_name.empty',
  'user.last_name.tooLong',
  'user.phone_number.empty',
  'user.phone_number.invalidFormat',
] as const satisfies readonly string[]

/**
 * Render an issue through the caller's translator.
 *
 * Takes only `key` and `params`, not the full `ValidationIssue`: the server
 * reports the same violation without a form-field `path`, and both sources
 * must resolve through the same catalog.
 */
export function translateIssue(issue: TranslatableIssue, translate: IssueTranslator): string {
  const message = translate(issue.key, issue.params)
  return message ? message : issue.key
}
