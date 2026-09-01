import type { IssueTranslator, TranslatableIssue } from './types'

/**
 * Every key the Rust validators can emit, in catalog order.
 *
 * The texts live in the consuming app's `validation` namespace; this package
 * holds keys only. A test in the app checks the catalog against this list, so
 * a new Rust variant cannot arrive without a translation.
 */
export const VALIDATION_KEYS = [
  // Tree
  'tree.species.empty',
  'tree.species.tooShort',
  'tree.species.tooLong',
  'tree.number.empty',
  'tree.number.tooShort',
  'tree.number.tooLong',
  'tree.planting_year.outOfRange',
  'tree.planting_year.invalidFormat',

  // Coordinate
  'coordinate.latitude.outOfRange',
  'coordinate.longitude.outOfRange',
  'coordinate.latitude.invalidFormat',
  'coordinate.longitude.invalidFormat',

  // Cluster
  'cluster.name.empty',
  'cluster.name.tooShort',
  'cluster.name.tooLong',
  'cluster.address.empty',
  'cluster.address.tooShort',
  'cluster.address.tooLong',
  'cluster.soil_condition.invalidFormat',

  // Vehicle
  'vehicle.number_plate.empty',
  'vehicle.number_plate.tooShort',
  'vehicle.number_plate.tooLong',
  'vehicle.model.empty',
  'vehicle.model.tooShort',
  'vehicle.model.tooLong',
  'water_capacity.outOfRange',
  'vehicle.water_capacity.invalidFormat',
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

  // Watering plan
  'watering_plan.cluster_ids.empty',
  'watering_plan.driver_ids.empty',
  'watering_plan.transporter_id.empty',
  'watering_plan.transporter_id.outOfRange',
  'watering_plan.trailer_id.outOfRange',
  'watering_plan.driver_ids.invalidFormat',
  'watering_plan.status.invalidFormat',
  'watering_plan.date.outOfRange',
  'watering_plan.start_point_name.empty',

  // Provenance. Only reachable through the server's validation block, not the
  // in-browser validators: a provider is set by an import, never typed in.
  'provenance.provider.empty',
  'provenance.provider.tooLong',

  // User
  'user.email.empty',
  'user.email.invalidFormat',
  'user.username.empty',
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
