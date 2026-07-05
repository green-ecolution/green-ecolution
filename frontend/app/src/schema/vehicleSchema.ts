import type { VehicleForm as VehicleFormBase } from '@green-ecolution/domain-wasm'
import { DrivingLicense, VehicleStatus, VehicleType } from '@green-ecolution/backend-client'

export type VehicleForm = Omit<VehicleFormBase, 'type' | 'drivingLicense' | 'status'> & {
  type: VehicleType
  drivingLicense: DrivingLicense
  status: VehicleStatus
}
