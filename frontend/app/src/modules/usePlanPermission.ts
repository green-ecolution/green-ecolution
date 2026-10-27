import type { Action } from '@/lib/auth/permissions'
import { useHasPermission } from '@/lib/auth/useHasPermission'
import type { ModulePlanning } from './types'

/** False without a planning module: an empty requirement would let everyone through. */
export function usePlanPermission(planning: ModulePlanning | undefined, action: Action): boolean {
  const allowed = useHasPermission(planning ? [`${planning.planResource}:${action}`] : [])
  return !!planning && allowed
}
