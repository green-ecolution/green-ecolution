import {
  draftResolver,
  validateVehicleDraft,
  type IssueTranslator,
  type ValidationIssue,
} from '@green-ecolution/domain-wasm'
import { DrivingLicense, VehicleAvailability, VehicleType } from '@green-ecolution/backend-client'
import type { FieldValues, Resolver } from 'react-hook-form'
import type { ModuleCapacityKind } from '@/modules/types'

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
  /** Every stored kind plus every offered one; only the offered ones are fields. */
  capacities: Record<string, number | undefined>
  description: string
}

type DraftInput = Record<string, unknown> & { capacities?: Record<string, unknown> }

/**
 * The core draft sees only the offered kinds, so a stored kind no module offers
 * neither satisfies nor breaks "at least one"; each offered amount then goes
 * through its module's own range check.
 */
export const vehicleDraftResolver = <TForm extends FieldValues>(
  translate: IssueTranslator,
  offered: readonly ModuleCapacityKind[],
): Resolver<TForm> =>
  draftResolver<TForm>((input) => {
    const { capacities = {}, ...rest } = input as DraftInput
    const ownIssues = validateVehicleDraft({
      ...rest,
      capacities: Object.fromEntries(offered.map(({ kind }) => [kind, capacities[kind]])),
    }) as ValidationIssue[]
    const moduleIssues = offered.flatMap(({ kind, validate }) => {
      const amount = capacities[kind]
      const issue = typeof amount === 'number' ? validate(amount) : null
      return issue ? [{ ...issue, path: `capacities.${kind}` }] : []
    })
    return [...ownIssues, ...moduleIssues]
  }, translate)
