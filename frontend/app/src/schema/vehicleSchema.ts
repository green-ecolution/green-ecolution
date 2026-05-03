import { DrivingLicense, VehicleStatus, VehicleType } from '@green-ecolution/backend-client'

export interface VehicleForm {
  numberPlate: string
  model: string
  type: VehicleType
  drivingLicense: DrivingLicense
  status: VehicleStatus
  height: number
  width: number
  length: number
  weight: number
  waterCapacity: number
  description: string
}
