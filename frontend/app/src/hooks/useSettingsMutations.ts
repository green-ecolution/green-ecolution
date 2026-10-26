import { useMutation, useQueryClient } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'
import type {
  IrrigationSettingsUpdateRequest,
  OrganizationSettingsUpdateRequest,
} from '@green-ecolution/backend-client'
import { settingsApi } from '@/api/backendApi'
import { irrigationApi } from '../../../modules/irrigation/src/api/client'
import createToast from '@/hooks/createToast'
import { statusOf } from '@/lib/httpError'

export interface UpdateSettingsVariables {
  orgId: string
  /** Core settings; left out when only irrigation values changed. */
  body?: OrganizationSettingsUpdateRequest
  /** Irrigation settings travel to their own endpoint. */
  irrigationBody?: IrrigationSettingsUpdateRequest
}

export const useSettingsMutations = () => {
  const queryClient = useQueryClient()
  const showToast = createToast()
  const { t } = useTranslation('settings')

  const updateSettings = useMutation({
    // Both requests go out together and the person hears about one outcome.
    mutationFn: async ({ orgId, body, irrigationBody }: UpdateSettingsVariables) => {
      const results = await Promise.allSettled([
        body
          ? settingsApi.updateOrganizationSettings({
              orgId,
              organizationSettingsUpdateRequest: body,
            })
          : undefined,
        irrigationBody
          ? irrigationApi.updateIrrigationSettings({
              orgId,
              irrigationSettingsUpdateRequest: irrigationBody,
            })
          : undefined,
      ])
      const failed = results.find((result) => result.status === 'rejected')
      if (failed) throw failed.reason
    },
    // Also after a failure: one of the two requests may have gone through.
    onSettled: (_data, _error, { orgId }) => {
      void queryClient.invalidateQueries({ queryKey: ['organizations', orgId, 'settings'] })
    },
    onSuccess: () => {
      // Saving any organization in the caller's own chain moves the viewport
      // their map opens at, and the answer is cached until told otherwise.
      void queryClient.invalidateQueries({ queryKey: ['users', 'me', 'map-view'] })
      showToast(t('organization.settings.toast.saved'))
    },
    onError: (error) => {
      showToast(
        statusOf(error) === 409
          ? t('organization.settings.toast.lockedByAncestor')
          : t('organization.settings.toast.saveFailed'),
        'error',
      )
    },
  })

  return { updateSettings }
}
