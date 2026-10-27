import {
  EvaluationApi,
  IrrigationApi,
  TreeClustersApi,
  TreesApi,
} from '@green-ecolution/backend-client'
import { configuration } from '@/api/backendApi'

export const treeApi = new TreesApi(configuration)
export const clusterApi = new TreeClustersApi(configuration)
export const evaluationApi = new EvaluationApi(configuration)
export const irrigationApi = new IrrigationApi(configuration)

export type { TreeResponse as Tree } from '@green-ecolution/backend-client'
export type { TreeCreateRequest as TreeCreate } from '@green-ecolution/backend-client'
export type { TreeUpdateRequest as TreeUpdate } from '@green-ecolution/backend-client'
export type { TreeClusterResponse as TreeCluster } from '@green-ecolution/backend-client'
export type { TreeClusterInListResponse as TreeClusterInList } from '@green-ecolution/backend-client'
export type { TreeClusterCreateRequest as TreeClusterCreate } from '@green-ecolution/backend-client'
export type { TreeClusterUpdateRequest as TreeClusterUpdate } from '@green-ecolution/backend-client'
export type { EvaluationResponse as Evaluation } from '@green-ecolution/backend-client'
export type { NearestTreeListResponse as NearestTreeList } from '@green-ecolution/backend-client'
export type { TreeWithDistanceResponse as TreeWithDistance } from '@green-ecolution/backend-client'
