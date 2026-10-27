import { CORE_RESOURCES, readPermissionsOf } from '@/lib/auth/permissions'
import type { ModuleResource } from '@/modules/types'

declare module '@/lib/auth/permissions' {
  interface ModuleResourceKeys {
    tree: true
    tree_cluster: true
    watering_plan: true
  }
}

export const irrigationResources: readonly ModuleResource[] = [
  {
    key: 'tree',
    area: 'greenspaces',
    textsKey: 'irrigation:settings.permission.resource.tree',
  },
  {
    key: 'tree_cluster',
    area: 'greenspaces',
    textsKey: 'irrigation:settings.permission.resource.tree_cluster',
  },
  {
    key: 'watering_plan',
    area: 'operations',
    textsKey: 'irrigation:settings.permission.resource.watering_plan',
  },
]

/** The evaluation spans core and irrigation data, so any read right of either opens it. */
export const EVALUATION_READ = readPermissionsOf([
  ...CORE_RESOURCES,
  ...irrigationResources.map(({ key }) => key),
])
