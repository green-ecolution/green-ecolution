export interface ValidationIssue {
  path: string
  field: string
  key: string
  params: Record<string, string | number>
}

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
  startPointName?: string
}
