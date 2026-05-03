import type { FieldError, FieldErrors, FieldValues, Resolver, ResolverResult } from 'react-hook-form'
import { translateIssue } from './messages'
import type { ValidationIssue } from './types'
import {
  validateTreeClusterDraft,
  validateTreeDraft,
  validateVehicleDraft,
  validateWateringPlanDraft,
} from '../pkg/domain_wasm.js'

type RawValidator = (input: unknown) => ValidationIssue[]

function makeResolver<TForm extends FieldValues>(validate: RawValidator): Resolver<TForm> {
  return async (values): Promise<ResolverResult<TForm>> => {
    const issues = validate(values)
    if (issues.length === 0) {
      return { values, errors: {} }
    }
    const errors: Record<string, FieldError> = {}
    for (const issue of issues) {
      errors[issue.path] = {
        type: issue.key,
        message: translateIssue(issue),
      }
    }
    // RHF's FieldErrors<TForm> uses statically-keyed types; runtime issue.path
    // is dynamic, so the structurally-equivalent record needs one boundary cast.
    return { values: {}, errors: errors as FieldErrors<TForm> }
  }
}

// wasm-bindgen emits `(input: any) => any` for these exports; tighten the
// return type at the boundary so the rest of the module is fully typed.
const treeValidator = validateTreeDraft as RawValidator
const clusterValidator = validateTreeClusterDraft as RawValidator
const vehicleValidator = validateVehicleDraft as RawValidator
const wateringPlanValidator = validateWateringPlanDraft as RawValidator

export const treeDraftResolver = <TForm extends FieldValues>(): Resolver<TForm> =>
  makeResolver<TForm>(treeValidator)

export const clusterDraftResolver = <TForm extends FieldValues>(): Resolver<TForm> =>
  makeResolver<TForm>(clusterValidator)

export const vehicleDraftResolver = <TForm extends FieldValues>(): Resolver<TForm> =>
  makeResolver<TForm>(vehicleValidator)

export const wateringPlanDraftResolver = <TForm extends FieldValues>(): Resolver<TForm> =>
  makeResolver<TForm>(wateringPlanValidator)
