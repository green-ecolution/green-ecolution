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
 * The core draft checks the offered kinds plus every stored amount of a kind
 * that is not offered: a vehicle that already has a capacity stays saveable
 * after its module is switched off. Each offered amount then goes through its
 * module's own range check.
 */
export const vehicleDraftResolver = <TForm extends FieldValues>(
  translate: IssueTranslator,
  offered: readonly ModuleCapacityKind[],
): Resolver<TForm> =>
  draftResolver<TForm>((input) => {
    const { capacities = {}, ...rest } = input as DraftInput
    const offeredKinds = new Set(offered.map(({ kind }) => kind))
    const storedElsewhere = Object.entries(capacities).filter(
      ([kind, amount]) => !offeredKinds.has(kind) && typeof amount === 'number',
    )
    const ownIssues = validateVehicleDraft({
      ...rest,
      capacities: Object.fromEntries<unknown>([
        ...offered.map(({ kind }): [string, unknown] => [kind, capacities[kind]]),
        ...storedElsewhere,
      ]),
    }) as ValidationIssue[]
    const moduleIssues = offered.flatMap(({ kind, validate }) => {
      const amount = capacities[kind]
      const issue = typeof amount === 'number' ? validate(amount) : null
      return issue ? [{ ...issue, path: `capacities.${kind}` }] : []
    })
    return [...ownIssues, ...moduleIssues]
  }, translate)
