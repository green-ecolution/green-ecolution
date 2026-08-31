export interface ValidationIssue {
  path: string
  field: string
  key: string
  params: Record<string, string | number>
}

/**
 * What the backend sends for a field violation, in `ErrorBody.validation`.
 *
 * Same `key` and `params` as the in-browser issue, built from the same Rust
 * constructor. It has no `path`, because the form-field path only exists in
 * the browser.
 */
export interface ServerValidationIssue {
  field: string
  key: string
  params: Record<string, string | number>
}

/** The minimum needed to render an issue: which rule broke, with what values. */
export type TranslatableIssue = Pick<ValidationIssue, 'key' | 'params'>

export interface TreeForm {
  number: string
  species: string
  plantingYear: number
  latitude: number
  longitude: number
  description: string
  treeClusterId: string | null
  sensorId: string | null
  provider?: string
}

export interface TreeclusterForm {
  name: string
  address: string
  description: string
  soilCondition: string
  treeIds: string[]
  /** Owning organization. Only sent on create; the backend defaults to the caller's org. */
  organizationId?: string
}

export interface VehicleForm {
  numberPlate: string
  model: string
  type: string
  drivingLicense: string
  status: string
  height: number
  width: number
  length: number
  weight: number
  waterCapacity: number
  description: string
}

export interface WateringPlanForm {
  date: Date
  status: string
  transporterId: string
  trailerId?: string
  driverIds: string[]
  clusterIds: string[]
  description: string
  startPointName: string
}
