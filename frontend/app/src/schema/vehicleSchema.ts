import {
  draftResolver,
  validateVehicleDraft,
  type IssueTranslator,
} from '@green-ecolution/domain-wasm'
import { DrivingLicense, VehicleAvailability, VehicleType } from '@green-ecolution/backend-client'
import type { FieldValues, Resolver } from 'react-hook-form'

export interface VehicleForm {
  numberPlate: string
  model: string
  type: VehicleType
  drivingLicense: DrivingLicense
  availability: VehicleAvailability
  height: number
  width: number
  length: number
  weight: number
  waterCapacity?: number
  cargoVolumeCapacity?: number
  description: string
}

export const vehicleDraftResolver = <TForm extends FieldValues>(
  translate: IssueTranslator,
): Resolver<TForm> => draftResolver<TForm>(validateVehicleDraft, translate)
