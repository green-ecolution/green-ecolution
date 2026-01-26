import { WateringPlanStatus } from '@green-ecolution/backend-client'
import { z } from 'zod'

export const wateringPlanSchemaBase = z.object({
  date: z.coerce.date(),
  status: z
    .nativeEnum(WateringPlanStatus)
    .refine((value) => Object.values(WateringPlanStatus).includes(value), {
      message: 'Kein korrekter Status.',
    }),
  transporterId: z.coerce.number().int().positive(),
  trailerId: z.coerce.number().int().positive().optional(),
  driverIds: z.array(z.string().uuid()),
  clusterIds: z.array(z.number()),
  description: z.string(),
})

const startOfToday = () => {
  const today = new Date()
  today.setHours(0, 0, 0, 0)
  return today
}

export const wateringPlanSchema = wateringPlanSchemaBase.extend({
  date: z.coerce.date().refine((date) => date >= startOfToday(), {
    message: 'Datum muss heute oder in der Zukunft liegen',
  }),
  driverIds: z
    .array(z.string().uuid())
    .min(1, { message: 'Es muss mindestens ein Mitwarbeiter ausgewählt werden' }),
  clusterIds: z.array(z.number()).min(1),
})

export const wateringPlanFinishedSchema = z.object({
  evaluation: z.array(
    z.object({
      consumedWater: z.number().positive(),
      treeClusterId: z.number(),
      wateringPlanId: z.number(),
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

export type WateringPlanForm = z.infer<typeof wateringPlanSchema>
export type WateringPlanFinishedForm = z.infer<typeof wateringPlanFinishedSchema>
export type WateringPlanCancelForm = z.infer<typeof wateringPlanCancelSchema>
export type WateringPlanOtherStatusUpdateForm = z.infer<typeof wateringPlanOtherStatusSchema>
