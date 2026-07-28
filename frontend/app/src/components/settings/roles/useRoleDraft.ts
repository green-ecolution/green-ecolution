import { useCallback, useState } from 'react'
import type { Role } from '@/api/backendApi'
import type { Permission, Permissions, Resource } from '@/lib/auth/permissions'
import {
  applyLevelWithinGrantable,
  clampToGrantable,
  toggleAction,
  type AccessLevel,
} from '@/lib/auth/permissionAreas'

export interface RoleDraft {
  kind: 'new' | 'existing'
  id?: string
  name: string
  description: string
  /** The role's real set, so permissions this UI does not render survive a save. */
  permissions: Set<string>
  clampedAway: string[]
}

interface Baseline {
  name: string
  description: string
  permissions: string
}

const canonical = (permissions: Iterable<string>): string => [...permissions].sort().join(',')

const baselineOf = (role: Role): Baseline => ({
  name: role.name,
  description: role.description ?? '',
  permissions: canonical(role.permissions),
})

export const useRoleDraft = (grantable: Permissions) => {
  const [draft, setDraft] = useState<RoleDraft | null>(null)
  const [baseline, setBaseline] = useState<Baseline | null>(null)

  const editExisting = useCallback((role: Role) => {
    setDraft({
      kind: 'existing',
      id: role.id,
      name: role.name,
      description: role.description ?? '',
      permissions: new Set(role.permissions),
      clampedAway: [],
    })
    setBaseline(baselineOf(role))
  }, [])

  const startNew = useCallback(() => {
    setDraft({ kind: 'new', name: '', description: '', permissions: new Set(), clampedAway: [] })
    setBaseline(null)
  }, [])

  const startCopy = useCallback(
    (source: Role) => {
      const { permissions, removed } = clampToGrantable(new Set(source.permissions), grantable)
      setDraft({
        kind: 'new',
        name: `${source.name} (Kopie)`,
        description: `Kopie von ${source.name}`,
        permissions,
        clampedAway: removed,
      })
      setBaseline(null)
    },
    [grantable],
  )

  const discard = useCallback(() => {
    setDraft(null)
    setBaseline(null)
  }, [])

  const setName = useCallback((name: string) => {
    setDraft((current) => (current ? { ...current, name } : current))
  }, [])

  const setDescription = useCallback((description: string) => {
    setDraft((current) => (current ? { ...current, description } : current))
  }, [])

  const setLevel = useCallback(
    (resource: Resource, level: AccessLevel) => {
      setDraft((current) =>
        current
          ? {
              ...current,
              permissions: applyLevelWithinGrantable(current.permissions, resource, level, grantable),
            }
          : current,
      )
    },
    [grantable],
  )

  const toggle = useCallback((permission: Permission) => {
    setDraft((current) =>
      current ? { ...current, permissions: toggleAction(current.permissions, permission) } : current,
    )
  }, [])

  const dirty =
    draft !== null &&
    (draft.name !== baseline?.name ||
      draft.description !== baseline?.description ||
      canonical(draft.permissions) !== baseline?.permissions)

  return {
    draft,
    dirty,
    editExisting,
    startNew,
    startCopy,
    discard,
    setName,
    setDescription,
    setLevel,
    toggle,
  }
}
