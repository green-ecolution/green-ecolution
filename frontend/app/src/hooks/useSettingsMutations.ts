import { useState } from 'react'
import { useMutation, useQueryClient } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'
import type { OrganizationSettingsUpdateRequest } from '@green-ecolution/backend-client'
import { moduleApi, settingsApi } from '@/api/backendApi'
import { modulesQuery, organizationModulesQuery } from '@/api/queries'
import createToast from '@/hooks/createToast'
import { statusOf } from '@/lib/httpError'

export interface UpdateSettingsVariables {
  orgId: string
  body: OrganizationSettingsUpdateRequest
}

export interface UpdateOrganizationModulesVariables {
  orgId: string
  /** `null` returns a module to inheritance; a missing key stays as it is. */
  modules: Record<string, boolean | null>
}

/** No toasts here: the page saves several parts at once and reports them through `useSettingsSave`. */
export const useSettingsMutations = () => {
  const queryClient = useQueryClient()

  const updateSettings = useMutation({
    mutationFn: ({ orgId, body }: UpdateSettingsVariables) =>
      settingsApi.updateOrganizationSettings({ orgId, organizationSettingsUpdateRequest: body }),
    onSettled: (_data, _error, { orgId }) => {
      void queryClient.invalidateQueries({ queryKey: ['organizations', orgId, 'settings'] })
    },
    onSuccess: () => {
      // Saving any organization in the caller's own chain moves the viewport
      // their map opens at, and the answer is cached until told otherwise.
      void queryClient.invalidateQueries({ queryKey: ['users', 'me', 'map-view'] })
    },
  })

  return { updateSettings }
}

export const useUpdateOrganizationModules = () => {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: ({ orgId, modules }: UpdateOrganizationModulesVariables) =>
      moduleApi.updateOrganizationModules({
        orgId,
        organizationModulesUpdateRequest: { modules },
      }),
    onSettled: (_data, _error, { orgId }) => {
      void queryClient.invalidateQueries({ queryKey: organizationModulesQuery(orgId).queryKey })
      // Navigation, route guards and module sections all follow the `modules` root.
      void queryClient.invalidateQueries({ queryKey: modulesQuery().queryKey })
    },
  })
}

export type SettingsSavePart = () => Promise<unknown>

/**
 * Saves the parts of the organization settings behind one button and reports a
 * single outcome. Phases run one after another, the parts within a phase
 * together; a failed part neither stops the others nor the next phase.
 */
export const useSettingsSave = () => {
  const showToast = createToast()
  const { t } = useTranslation('settings')
  const [pending, setPending] = useState(false)

  const save = async (phases: readonly (readonly SettingsSavePart[])[]) => {
    if (phases.every((parts) => parts.length === 0)) return
    setPending(true)
    const failures: unknown[] = []
    for (const parts of phases) {
      const results = await Promise.allSettled(parts.map((part) => part()))
      for (const result of results) {
        if (result.status === 'rejected') failures.push(result.reason)
      }
    }
    setPending(false)

    if (failures.length === 0) {
      showToast(t('organization.settings.toast.saved'))
      return
    }
    showToast(
      failures.some((error) => statusOf(error) === 409)
        ? t('organization.settings.toast.lockedByAncestor')
        : t('organization.settings.toast.saveFailed'),
      'error',
    )
  }

  return { save, pending }
}
