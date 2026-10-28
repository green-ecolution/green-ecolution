import type { ModuleResource } from '@/modules/types'

declare module '@/lib/auth/permissions' {
  interface ModuleResourceKeys {
    waste_bin: true
    collection_plan: true
  }
}

export const wasteResources: readonly ModuleResource[] = [
  {
    key: 'waste_bin',
    area: 'operations',
    textsKey: 'waste:settings.permission.resource.waste_bin',
  },
  {
    key: 'collection_plan',
    area: 'operations',
    textsKey: 'waste:settings.permission.resource.collection_plan',
  },
]
