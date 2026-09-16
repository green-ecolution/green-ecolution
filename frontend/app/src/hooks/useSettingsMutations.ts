import { useMutation, useQueryClient } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'
import type { OrganizationSettingsUpdateRequest } from '@green-ecolution/backend-client'
import { settingsApi } from '@/api/backendApi'
import createToast from '@/hooks/createToast'
import { statusOf } from '@/lib/httpError'

export interface UpdateSettingsVariables {
  orgId: string
  body: OrganizationSettingsUpdateRequest
}

export const useSettingsMutations = () => {
  const queryClient = useQueryClient()
  const showToast = createToast()
  const { t } = useTranslation('settings')

  const updateSettings = useMutation({
    mutationFn: ({ orgId, body }: UpdateSettingsVariables) =>
      settingsApi.updateOrganizationSettings({
        orgId,
        organizationSettingsUpdateRequest: body,
      }),
    onSuccess: (_data, { orgId }) => {
      void queryClient.invalidateQueries({ queryKey: ['organizations', orgId, 'settings'] })
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
