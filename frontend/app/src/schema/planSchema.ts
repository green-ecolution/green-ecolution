import {
  draftResolver,
  validatePlanDraft,
  type IssueTranslator,
} from '@green-ecolution/domain-wasm'
import { PlanStatus } from '@green-ecolution/backend-client'
import type { FieldValues, Resolver } from 'react-hook-form'
import { z } from 'zod'

export interface PlanForm {
  date: Date
  status: PlanStatus
  transporterId: string
  trailerId?: string
  transporterTankUsed: boolean
  driverIds: string[]
  stopIds: string[]
  description: string
  startPointName: string
  // Not part of the domain draft: it picks the module the plan runs in, and
  // the backend checks it against the active modules.
  module: string
}

export const planDraftResolver = <TForm extends FieldValues>(
  translate: IssueTranslator,
): Resolver<TForm> => draftResolver<TForm>(validatePlanDraft, translate)

// A state-machine action schema — it validates the input of a transition (not
// the draft shape), so it keeps its Zod schema until a domain validator exists.
export const planCancelSchema = z.object({
  cancellationNote: z.string().min(1),
})

export type PlanCancelForm = z.infer<typeof planCancelSchema>
