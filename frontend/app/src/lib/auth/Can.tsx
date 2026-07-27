import type { ReactNode } from 'react'
import type { PermissionRequirement } from './permissions'
import { useHasPermission } from './useHasPermission'

interface CanProps {
  permission: PermissionRequirement
  children: ReactNode
  /** Rendered instead of children when the user lacks the permission. */
  fallback?: ReactNode
}

/** Renders `children` only when the current user satisfies `permission`. */
export function Can({ permission, children, fallback = null }: CanProps): ReactNode {
  return useHasPermission(permission) ? children : fallback
}
