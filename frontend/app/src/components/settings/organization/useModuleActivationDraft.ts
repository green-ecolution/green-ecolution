import { useState } from 'react'
import { useQuery } from '@tanstack/react-query'
import type {
  OrganizationModulesResponse,
  OrganizationRef,
  SettingOriginDto,
} from '@/api/backendApi'
import { organizationModulesQuery } from '@/api/queries'
import { useUpdateOrganizationModules } from '@/hooks/useSettingsMutations'
import type { ModuleSettingsDraft } from '@/modules/types'

type OwnSwitches = Record<string, boolean | null>

export interface ModuleActivationRow {
  key: string
  /** The state the switch shows: the draft's own value, else the one in force. */
  enabled: boolean
  /** Stored origin; an unsaved flip has not changed it yet. */
  origin: SettingOriginDto
  sourceId: string | null
  /** This organization's own switch in the draft; `null` inherits. */
  own: boolean | null
  /** The parent that set this organization's switch; `null` when it set it itself. */
  setBy: string | null
  locked: boolean
}

export interface ModuleActivationDraft extends ModuleSettingsDraft {
  rows: ModuleActivationRow[]
  enforcedBy: OrganizationRef | null
  /** Activation in force on the server, which a section's visibility follows. */
  isActive: (key: string) => boolean
  setOwn: (key: string, value: boolean | null) => void
}

const ownSwitchesOf = (response: OrganizationModulesResponse): OwnSwitches =>
  Object.fromEntries(response.modules.map((module) => [module.key, module.ownValue ?? null]))

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
  // The edit remembers the response it was made on. Keyed on content: a
  // refetch that changed nothing leaves a flip alone, one that moved a switch
  // replaces it with the server's truth.
  const [edit, setEdit] = useState<{ key: string; own: OwnSwitches } | null>(null)
  const serverKey = fingerprint(organizationId, server)
  const own =
    server && serverKey !== null
      ? edit?.key === serverKey
        ? edit.own
        : ownSwitchesOf(server)
      : null

  const enforcedBy = server?.enforcedBy ?? null
  const ready = enabled && server !== undefined && own !== null

  const changes: OwnSwitches = {}
  if (ready) {
    for (const module of server.modules) {
      const drafted = own[module.key] ?? null
      if (drafted !== (module.ownValue ?? null)) changes[module.key] = drafted
    }
  }

  const rows: ModuleActivationRow[] = ready
    ? server.modules.map((module) => {
        const drafted = own[module.key] ?? null
        const setBy = module.setBy?.id ?? null
        return {
          key: module.key,
          enabled: drafted ?? module.enabled,
          origin: module.origin,
          sourceId: module.source?.id ?? null,
          own: drafted,
          setBy: setBy === organizationId ? null : setBy,
          locked: enforcedBy !== null || module.fixedByAncestor,
        }
      })
    : []

  const setOwn = (key: string, value: boolean | null) => {
    // The backend refuses the whole write under a lock above, and a switch a
    // parent set for this organization.
    const fixed = server?.modules.some((module) => module.key === key && module.fixedByAncestor)
    if (!own || serverKey === null || enforcedBy || fixed) return
    setEdit({ key: serverKey, own: { ...own, [key]: value } })
  }

  const reset = () => setEdit(null)

  const save = async () => {
    if (Object.keys(changes).length === 0) return
    await updateModules.mutateAsync({ orgId: organizationId, modules: changes })
  }

  return {
    loading: !ready,
    loadError: error,
    dirty: Object.keys(changes).length > 0,
    valid: true,
    reset,
    save,
    rows,
    enforcedBy,
    isActive: (key) =>
      server?.modules.some((module) => module.key === key && module.enabled) ?? false,
    setOwn,
  }
}
