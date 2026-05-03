import { SoilCondition } from '@green-ecolution/backend-client'

export interface TreeclusterForm {
  name: string
  address: string
  description: string
  soilCondition: SoilCondition
  treeIds: number[]
}
