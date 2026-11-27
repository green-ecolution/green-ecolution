import { Package } from 'lucide-react'
import { validateCapacityAmount, type ValidationIssue } from '@green-ecolution/domain-wasm'
import type { ModuleCapacityKind } from '@/modules/types'

export const CARGO_VOLUME = 'cargo_volume'

export const cargoVolumeCapacity: ModuleCapacityKind = {
  kind: CARGO_VOLUME,
  icon: Package,
  validate: (amount) => validateCapacityAmount(amount) as ValidationIssue | null,
  texts: {
    label: 'waste:capacity.label',
    vehicleField: 'waste:capacity.vehicleField',
    unit: 'waste:capacity.unit',
    stationAction: 'waste:capacity.stationAction',
    demand: 'waste:capacity.demand',
    transporter: 'waste:capacity.transporter',
    transporterUnused: 'waste:capacity.transporterUnused',
    transporterUsedFormLabel: 'waste:capacity.transporterUsedFormLabel',
    transporterUsedHint: 'waste:capacity.transporterUsedHint',
  },
}
