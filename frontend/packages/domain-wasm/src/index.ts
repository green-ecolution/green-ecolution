export type {
  ValidationIssue,
  TranslatableIssue,
  IssueTranslator,
  TreeForm,
  TreeclusterForm,
  VehicleForm,
  WateringPlanForm,
} from './types'
export { translateIssue, VALIDATION_KEYS } from './messages'

// Re-export all wasm-bindgen functions.
export {
  validateSpecies,
  validateTreeNumber,
  validatePlantingYear,
  plantingYearMin,
  plantingYearMax,
  plantingYearIsFuture,
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
  validateSensorId,
  validatePhoneNumber,
  validateWaterDemand,
  waterDemandMin,
  waterDemandMax,
  validateJustWateredTtlHours,
  justWateredTtlMinHours,
  justWateredTtlMaxHours,
  validateMapView,
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
