import { SoilCondition } from '@green-ecolution/backend-client'
import { z } from 'zod'

export const clusterSchemaBase = z.object({
  name: z.string(),
  address: z.string(),
  description: z.string(),
  soilCondition: z
    .nativeEnum(SoilCondition)
    .refine((value) => Object.values(SoilCondition).includes(value), {
      message: 'Keine korrekte Bodenbeschaffenheit.',
    }),
  treeIds: z.array(z.number().int()),
})

export const clusterSchema = clusterSchemaBase.extend({
  name: z.string().min(1, 'Name ist erforderlich.'),
  address: z.string().min(1, 'Adresse ist erforderlich.'),
})

export type TreeclusterForm = z.infer<typeof clusterSchema>
