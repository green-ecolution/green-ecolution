import type { TreeclusterForm as TreeclusterFormBase } from '@green-ecolution/domain-wasm'
import { SoilCondition } from '@green-ecolution/backend-client'

export type TreeclusterForm = Omit<TreeclusterFormBase, 'soilCondition'> & {
  soilCondition: SoilCondition
}
