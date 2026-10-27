import type { WateringPlanForm as WateringPlanFormBase } from '@green-ecolution/domain-wasm'
import { PlanStatus as WateringPlanStatus } from '@green-ecolution/backend-client'
import { z } from 'zod'

// `module` is not part of the domain draft: it picks the module the plan runs
// in, and the backend checks it against the active modules.
export type WateringPlanForm = Omit<WateringPlanFormBase, 'status'> & {
  status: WateringPlanStatus
  module: string
}

// State-machine action schemas — these validate transition inputs (not draft
// shape), so they keep their Zod schema until corresponding domain validators
// are added.
export const wateringPlanCancelSchema = z.object({
  cancellationNote: z.string().min(1),
})

export const wateringPlanOtherStatusSchema = z.object({
  status: z
    .nativeEnum(WateringPlanStatus)
    .refine((value) => Object.values(WateringPlanStatus).includes(value), {
      message: 'Kein korrekter Status.',
    }),
})

export type WateringPlanCancelForm = z.infer<typeof wateringPlanCancelSchema>
export type WateringPlanOtherStatusUpdateForm = z.infer<typeof wateringPlanOtherStatusSchema>
