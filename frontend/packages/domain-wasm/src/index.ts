export type {
  ValidationIssue,
  TreeForm,
  TreeclusterForm,
  VehicleForm,
  WateringPlanForm,
} from './types'
export { translateIssue } from './messages'

// Re-export all wasm-bindgen functions.
export {
  validateSpecies,
  validateTreeNumber,
  validatePlantingYear,
  validateCoordinate,
  validateClusterName,
  validateClusterAddress,
  validateRegionName,
  validateNumberPlate,
  validateVehicleModel,
  validateVehicleDimension,
  validateWaterCapacity,
  validateDistance,
  validateEmail,
  validateUsername,
  licenseSatisfies,
  validateSensorId,
  validateTreeDraft,
  validateTreeClusterDraft,
  validateVehicleDraft,
  validateWateringPlanDraft,
} from '../pkg/domain_wasm.js'

// Resolvers
export {
  treeDraftResolver,
  clusterDraftResolver,
  vehicleDraftResolver,
  wateringPlanDraftResolver,
} from './resolver'
