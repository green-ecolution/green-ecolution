import { SoilCondition } from '@green-ecolution/backend-client'

export interface TreeclusterForm {
  name: string
  address: string
  description: string
  soilCondition: SoilCondition
  treeIds: string[]
  /** Owning organization. Only sent on create; the backend defaults to the caller's org. */
  organizationId?: string
}
