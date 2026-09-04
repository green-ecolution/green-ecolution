import type { VehicleForm as VehicleFormBase } from '@green-ecolution/domain-wasm'
import { DrivingLicense, VehicleAvailability, VehicleType } from '@green-ecolution/backend-client'

export type VehicleForm = Omit<VehicleFormBase, 'type' | 'drivingLicense' | 'availability'> & {
  type: VehicleType
  drivingLicense: DrivingLicense
  availability: VehicleAvailability
}
