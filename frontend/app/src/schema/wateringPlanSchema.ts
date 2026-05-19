import { WateringPlanStatus } from '@green-ecolution/backend-client'
import { z } from 'zod'

export interface WateringPlanForm {
  date: Date
  status: WateringPlanStatus
  transporterId: string
  trailerId?: string
  driverIds: string[]
  clusterIds: string[]
  description: string
}

// State-machine action schemas — these validate transition inputs (not draft
// shape), so they keep their Zod schema until corresponding domain validators
// are added.
export const wateringPlanFinishedSchema = z.object({
  evaluation: z.array(
    z.object({
      consumedWater: z.coerce.number().positive(),
      treeClusterId: z.string().uuid(),
      wateringPlanId: z.string().uuid(),
    }),
  ),
})

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

export type WateringPlanFinishedForm = z.infer<typeof wateringPlanFinishedSchema>
export type WateringPlanCancelForm = z.infer<typeof wateringPlanCancelSchema>
export type WateringPlanOtherStatusUpdateForm = z.infer<typeof wateringPlanOtherStatusSchema>
