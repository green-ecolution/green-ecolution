import type { FieldErrors, FieldValues, Resolver, ResolverResult } from 'react-hook-form'
import { translateIssue } from './messages'
import type { ValidationIssue } from './types'
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

export const treeDraftResolver = buildResolver(validateTreeDraft as RawValidator)
export const clusterDraftResolver = buildResolver(validateTreeClusterDraft as RawValidator)
export const vehicleDraftResolver = buildResolver(validateVehicleDraft as RawValidator)
export const wateringPlanDraftResolver = buildResolver(validateWateringPlanDraft as RawValidator)
