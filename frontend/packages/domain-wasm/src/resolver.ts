import type { FieldErrors, FieldValues, Resolver, ResolverResult } from 'react-hook-form'
import { translateIssue } from './messages'
import type { TreeForm, TreeclusterForm, ValidationIssue, VehicleForm, WateringPlanForm } from './types'
import {
  validateTreeClusterDraft,
  validateTreeDraft,
  validateVehicleDraft,
  validateWateringPlanDraft,
} from '../pkg/domain_wasm.js'

type RawValidator = (input: unknown) => ValidationIssue[]

function buildResolver<TForm extends FieldValues>(validate: RawValidator): Resolver<TForm> {
  return async (values): Promise<ResolverResult<TForm>> => {
    const issues = validate(values)
    if (issues.length === 0) {
      return { values, errors: {} }
    }
    const errors: FieldErrors<TForm> = {}
    for (const issue of issues) {
      // RHF FieldErrors uses any to allow string-keyed assignment.
      // Casting once is cleaner than fighting the type for unknown shapes.
      ;(errors as Record<string, { type: string; message: string }>)[issue.path] = {
        type: issue.key,
        message: translateIssue(issue),
      }
    }
    return { values: {}, errors }
  }
}

export const treeDraftResolver: Resolver<TreeForm> = buildResolver<TreeForm>(validateTreeDraft as RawValidator)
export const clusterDraftResolver: Resolver<TreeclusterForm> = buildResolver<TreeclusterForm>(validateTreeClusterDraft as RawValidator)
export const vehicleDraftResolver: Resolver<VehicleForm> = buildResolver<VehicleForm>(validateVehicleDraft as RawValidator)
export const wateringPlanDraftResolver: Resolver<WateringPlanForm> = buildResolver<WateringPlanForm>(validateWateringPlanDraft as RawValidator)
