import { z } from 'zod'

export const treeSchemaBase = z.object({
  latitude: z.number().min(-90).max(90),
  longitude: z.number().min(-180).max(180),
  number: z.string(),
  species: z.string(),
  provider: z.string().optional(),
  plantingYear: z
    .number()
    .int()
    .min(2020, 'Pflanzjahr vor 2020 ist nicht möglich.')
    .max(new Date().getFullYear(), 'Pflanzjahr kann nicht in der Zukunft liegen.'),
  treeClusterId: z.number(),
  sensorId: z.string(),
  description: z.string(),
})

export const treeSchema = treeSchemaBase.extend({
  number: z.string().min(1, 'Baumnummer ist erforderlich.'),
  species: z.string().min(1, 'Art ist erforderlich.'),
  treeClusterId: z.number().or(z.literal(-1)), // -1 no cluster selected
  sensorId: z.string().or(z.literal('-1')), // -1 no sensor selected
})

export type TreeForm = z.infer<typeof treeSchemaBase>
