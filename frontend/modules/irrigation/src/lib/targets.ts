import type { TargetRefDto } from '@green-ecolution/backend-client'

export const IRRIGATION = 'irrigation'
export const TREE_KIND = 'tree'
export const CLUSTER_KIND = 'cluster'

export const treeTarget = (treeId: string): TargetRefDto => ({
  module: IRRIGATION,
  kind: TREE_KIND,
  id: treeId,
})

export const isTreeTarget = (target?: TargetRefDto | null): target is TargetRefDto =>
  target?.module === IRRIGATION && target.kind === TREE_KIND

export const clusterTarget = (clusterId: string): TargetRefDto => ({
  module: IRRIGATION,
  kind: CLUSTER_KIND,
  id: clusterId,
})
