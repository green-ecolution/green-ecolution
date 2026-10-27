import { z } from 'zod'

export const clusterOutcomeSchema = z.object({
  outcomes: z.array(
    z.object({
      stop: z.string().min(1),
      amount: z.coerce.number().positive(),
    }),
  ),
})
