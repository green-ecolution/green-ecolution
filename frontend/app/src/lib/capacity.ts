import type { ComponentType } from 'react'
import { Droplet, Package } from 'lucide-react'
import type { CapacityKindDto } from '@green-ecolution/backend-client'

interface CapacityDisplay {
  icon: ComponentType<{ className?: string }>
  demandLabelKey: 'detail.demandLabel.water' | 'detail.demandLabel.cargo_volume'
}

/** Every capacity is measured in liters; the kind only changes the wording and the icon. */
export const capacityDisplay: Record<CapacityKindDto, CapacityDisplay> = {
  water: { icon: Droplet, demandLabelKey: 'detail.demandLabel.water' },
  cargo_volume: { icon: Package, demandLabelKey: 'detail.demandLabel.cargo_volume' },
}
