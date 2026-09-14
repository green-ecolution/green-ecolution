import { useState } from 'react'
import { toggleAction } from '@/lib/auth/permissionAreas'
import type { Permission } from '@/lib/auth/permissions'

const samePermissionSet = (a: readonly string[], b: readonly string[]): boolean => {
  const left = new Set(a)
  const right = new Set(b)
  return left.size === right.size && [...left].every((entry) => right.has(entry))
}

/**
 * Tracks the plugin's own permission set and the access (required) permission
 * set together: the access set mirrors the plugin set until it is edited
 * directly, at which point mirroring stops for good. The two sets answer
 * different questions — what the plugin itself may do, versus who may open
 * its view — and are deliberately independent afterwards: a plugin can hold
 * write rights (to import data) while its view only requires `read` (a
 * status page), and a plugin holding no rights at all can still require a
 * real permission to view its (possibly sensitive) data. So the access set
 * is never re-derived from the plugin set at submit time either — whatever
 * the admin last set for access is exactly what gets submitted.
 *
 * `initialAccess` diverging from `initialPermissions` is read as evidence
 * that a previous edit already ended the mirroring — there is no persisted
 * "touched" flag, so an existing divergence is the only signal available.
 */
export const usePluginPermissionDraft = (
  initialPermissions: readonly string[] = [],
  initialAccess: readonly string[] = [],
) => {
  const [permissions, setPermissions] = useState<ReadonlySet<string>>(
    () => new Set(initialPermissions),
  )
  const [accessPermissions, setAccessPermissions] = useState<ReadonlySet<string>>(
    () => new Set(initialAccess),
  )
  const [accessTouched, setAccessTouched] = useState(
    () => !samePermissionSet(initialPermissions, initialAccess),
  )

  const togglePermission = (permission: Permission) => {
    setPermissions((prev) => toggleAction(prev, permission))
    if (!accessTouched) {
      setAccessPermissions((prev) => toggleAction(prev, permission))
    }
  }

  const toggleAccessPermission = (permission: Permission) => {
    setAccessTouched(true)
    setAccessPermissions((prev) => toggleAction(prev, permission))
  }

  const reset = (nextPermissions: readonly string[] = [], nextAccess: readonly string[] = []) => {
    setPermissions(new Set(nextPermissions))
    setAccessPermissions(new Set(nextAccess))
    setAccessTouched(!samePermissionSet(nextPermissions, nextAccess))
  }

  return {
    permissions,
    accessPermissions,
    accessTouched,
    togglePermission,
    toggleAccessPermission,
    reset,
  }
}
