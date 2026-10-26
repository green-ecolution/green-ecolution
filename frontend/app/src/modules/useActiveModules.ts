import { useSuspenseQuery } from '@tanstack/react-query'
import { modulesQuery } from '@/api/queries'
import { activeManifestsFrom } from './activeModules'
import { moduleManifests } from './registry'
import type { ModuleManifest } from './types'

export interface ActiveModules {
  manifests: ModuleManifest[]
  isActive: (key: string) => boolean
}

export function useActiveModules(): ActiveModules {
  const { data } = useSuspenseQuery(modulesQuery())
  const manifests = activeManifestsFrom(moduleManifests, data)
  return { manifests, isActive: (key) => manifests.some((manifest) => manifest.key === key) }
}
