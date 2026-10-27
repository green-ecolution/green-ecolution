import type { ComponentType } from 'react'
import { Droplet, Package } from 'lucide-react'
import type { CapacityKindDto } from '@green-ecolution/backend-client'

interface CapacityDisplay {
  icon: ComponentType<{ className?: string }>
  demandLabelKey: 'detail.demandLabel.water' | 'detail.demandLabel.cargo_volume'
  transporterLabelKey: `detail.transporterCapacity.${CapacityKindDto}.label`
  transporterUsedValueKey: `detail.transporterCapacity.${CapacityKindDto}.used`
  transporterUnusedValueKey: `detail.transporterCapacity.${CapacityKindDto}.unused`
  transporterUsedFormLabelKey: `form.transporterCapacityUsed.${CapacityKindDto}.label`
  transporterUsedHintKey: `form.transporterCapacityUsed.${CapacityKindDto}.hint`
}

const transporterKeys = <K extends CapacityKindDto>(kind: K) => ({
  transporterLabelKey: `detail.transporterCapacity.${kind}.label` as const,
  transporterUsedValueKey: `detail.transporterCapacity.${kind}.used` as const,
  transporterUnusedValueKey: `detail.transporterCapacity.${kind}.unused` as const,
  transporterUsedFormLabelKey: `form.transporterCapacityUsed.${kind}.label` as const,
  transporterUsedHintKey: `form.transporterCapacityUsed.${kind}.hint` as const,
})

/** Every capacity is measured in liters; the kind only changes the wording and the icon. */
export const capacityDisplay: Record<CapacityKindDto, CapacityDisplay> = {
  water: {
    icon: Droplet,
    demandLabelKey: 'detail.demandLabel.water',
    ...transporterKeys('water'),
  },
  cargo_volume: {
    icon: Package,
    demandLabelKey: 'detail.demandLabel.cargo_volume',
    ...transporterKeys('cargo_volume'),
  },
}
