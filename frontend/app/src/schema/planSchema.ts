import type { PlanForm as PlanFormBase } from '@green-ecolution/domain-wasm'
import { PlanStatus } from '@green-ecolution/backend-client'
import { z } from 'zod'

// `module` is not part of the domain draft: it picks the module the plan runs
// in, and the backend checks it against the active modules.
export type PlanForm = Omit<PlanFormBase, 'status'> & {
  status: PlanStatus
  module: string
}

// A state-machine action schema — it validates the input of a transition (not
// the draft shape), so it keeps its Zod schema until a domain validator exists.
export const planCancelSchema = z.object({
  cancellationNote: z.string().min(1),
})

export type PlanCancelForm = z.infer<typeof planCancelSchema>
