import { useState } from 'react'
import { useQuery } from '@tanstack/react-query'
import type { OrganizationModulesResponse } from '@/api/backendApi'
import { organizationModulesQuery } from '@/api/queries'
import { useUpdateOrganizationModules } from '@/hooks/useSettingsMutations'

type Flips = Record<string, boolean>

export interface ModuleActivationRow {
  key: string
  enabled: boolean
  /** A parent set this switch; this organization cannot move it. */
  fixed: boolean
  /** The organization that set it, when it is a visible one. */
  fixedBy: string | null
}

export interface ModuleActivationDraft {
  loading: boolean
  loadError: unknown
  dirty: boolean
  reset: () => void
  rows: ModuleActivationRow[]
  /** The switch as drafted, which a card's fields follow. */
  isEnabled: (key: string) => boolean
  setEnabled: (key: string, enabled: boolean) => void
  enabling: boolean
  disabling: boolean
  saveEnabling: () => Promise<void>
  saveDisabling: () => Promise<void>
}

const fingerprint = (orgId: string, response: OrganizationModulesResponse | undefined) =>
  response ? JSON.stringify([orgId, response]) : null

export const useModuleActivationDraft = (
  organizationId: string,
  { enabled }: { enabled: boolean },
): ModuleActivationDraft => {
  const { data: server, error } = useQuery({
    ...organizationModulesQuery(organizationId),
    enabled,
  })
  const updateModules = useUpdateOrganizationModules()
  // The edit remembers the response it was made on: a refetch that changed
  // nothing leaves a flip alone, one that moved a switch replaces it.
  const [edit, setEdit] = useState<{ key: string; flips: Flips } | null>(null)
  const serverKey = fingerprint(organizationId, server)
  const flips = edit?.key === serverKey ? edit.flips : {}
  const modules = enabled && server ? server.modules : []

  // Giving up the own switch where the drafted state is what the organization
  // would inherit keeps it following its parent, as with the values.
  const changes = (direction: boolean): Record<string, boolean | null> =>
    Object.fromEntries(
      modules
        .filter((module) => flips[module.key] === direction && module.enabled !== direction)
        .map((module) => [module.key, module.inheritedEnabled === direction ? null : direction]),
    )
  const enablingChanges = changes(true)
  const disablingChanges = changes(false)
  const enabling = Object.keys(enablingChanges).length > 0
  const disabling = Object.keys(disablingChanges).length > 0

  const send = async (modulesToWrite: Record<string, boolean | null>) => {
    if (Object.keys(modulesToWrite).length === 0) return
    await updateModules.mutateAsync({ orgId: organizationId, modules: modulesToWrite })
  }

  return {
    loading: !enabled || server === undefined,
    loadError: error,
    dirty: enabling || disabling,
    reset: () => setEdit(null),
    rows: modules.flatMap((module) => {
      const fixed = module.fixedByAncestor
      // Switched off from above: nothing this organization could do with it.
      if (fixed && !module.enabled) return []
      return {
        key: module.key,
        enabled: flips[module.key] ?? module.enabled,
        fixed,
        fixedBy: fixed ? (module.setBy?.id ?? null) : null,
      }
    }),
    isEnabled: (key) => {
      const module = modules.find((candidate) => candidate.key === key)
      return module ? (flips[key] ?? module.enabled) : false
    },
    setEnabled: (key, value) => {
      const module = modules.find((candidate) => candidate.key === key)
      if (!module || module.fixedByAncestor || serverKey === null) return
      setEdit({ key: serverKey, flips: { ...flips, [key]: value } })
    },
    enabling,
    disabling,
    saveEnabling: () => send(enablingChanges),
    saveDisabling: () => send(disablingChanges),
  }
}
