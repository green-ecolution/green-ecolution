import { Droplet } from 'lucide-react'
import { validateCapacityAmount, type ValidationIssue } from '@green-ecolution/domain-wasm'
import type { ModuleCapacityKind } from '@/modules/types'

export const WATER = 'water'

export const waterCapacity: ModuleCapacityKind = {
  kind: WATER,
  icon: Droplet,
  validate: (amount) => validateCapacityAmount(amount) as ValidationIssue | null,
  texts: {
    label: 'irrigation:capacity.label',
    vehicleField: 'irrigation:capacity.vehicleField',
    unit: 'irrigation:capacity.unit',
    stationAction: 'irrigation:capacity.stationAction',
    demand: 'irrigation:capacity.demand',
    transporter: 'irrigation:capacity.transporter',
    transporterUnused: 'irrigation:capacity.transporterUnused',
    transporterUsedFormLabel: 'irrigation:capacity.transporterUsedFormLabel',
    transporterUsedHint: 'irrigation:capacity.transporterUsedHint',
  },
}
