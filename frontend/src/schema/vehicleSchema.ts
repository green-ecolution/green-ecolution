import { VehicleType, DrivingLicense, VehicleStatus } from '@green-ecolution/backend-client'
import { z } from 'zod'

export const vehicleSchema = z.object({
  numberPlate: z.string().min(1, 'Kennzeichen ist erforderlich.'),
  model: z.string().min(1, 'Modell ist erforderlich.'),
  type: z.nativeEnum(VehicleType, { message: 'Kein korrekter Fahrzeugtyp.' }),
  drivingLicense: z.nativeEnum(DrivingLicense, { message: 'Keine korrekte Fahrzeugerlaubnis.' }),
  status: z
    .nativeEnum(VehicleStatus)
    .refine((value) => Object.values(VehicleStatus).includes(value), {
      message: 'Keine korrekter Fahrzeugstatus.',
    }),
  height: z.coerce.number().positive().min(1, 'Höhe ist erforderlich.'),
  width: z.coerce.number().min(1, 'Breite ist erforderlich.'),
  length: z.coerce.number().min(1, 'Länge ist erforderlich.'),
  weight: z.coerce.number().min(1, 'Gewicht ist erforderlich.'),
  waterCapacity: z.coerce.number().min(1, 'Wasserkapazität ist erforderlich.').gte(80, {
    message: 'Kapazität muss mindestens 80 Liter betragen und eine ganze Zahl sein.',
  }),
  description: z.string(),
})

export type VehicleForm = z.infer<typeof vehicleSchema>
