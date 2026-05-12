import type {
  FieldError,
  FieldErrors,
  FieldValues,
  Resolver,
  ResolverResult,
} from 'react-hook-form'
import { translateIssue } from './messages'
import type { ValidationIssue } from './types'
import {
  validateTreeClusterDraft,
  validateTreeDraft,
  validateVehicleDraft,
  validateWateringPlanDraft,
} from '../pkg/domain_wasm.js'

type RawValidator = (input: unknown) => ValidationIssue[]

// JS-side normalisation before crossing the WASM boundary. Domain DTOs use
// strict serde types (RFC 3339 strings for dates), but RHF stores `Date`
// objects. Strict deserialization would throw before validation runs, so we
// shape the values into wire-format types here.
function normaliseForWasm(values: unknown): unknown {
  if (values instanceof Date) return values.toISOString()
  if (Array.isArray(values)) return values.map(normaliseForWasm)
  if (values !== null && typeof values === 'object') {
    const out: Record<string, unknown> = {}
    for (const [k, v] of Object.entries(values)) {
      out[k] = normaliseForWasm(v)
    }
    return out
  }
  return values
}

function makeResolver<TForm extends FieldValues>(validate: RawValidator): Resolver<TForm> {
  return async (values): Promise<ResolverResult<TForm>> => {
    const issues = validate(normaliseForWasm(values))
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
