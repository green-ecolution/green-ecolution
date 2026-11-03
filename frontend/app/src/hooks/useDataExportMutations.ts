import { useMutation, useQueryClient } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'
import { dataExportApi } from '@/api/backendApi'
import {
  toDataExportRequest,
  toDataExportWireBody,
  type DataExportFormValues,
} from '@/api/dataExport'
import createToast from '@/hooks/createToast'
import { resolveApiError } from '@/lib/apiError'

export const useDataExportMutations = (orgId: string) => {
  const queryClient = useQueryClient()
  const showToast = createToast()
  const { t } = useTranslation('settings')

  const refresh = () =>
    queryClient.invalidateQueries({ queryKey: ['organizations', orgId, 'data-export'] })

  const showFailure = (error: unknown) => {
    void resolveApiError(error).then((info) => showToast(info.message, 'error'))
  }

  const save = useMutation({
    mutationFn: (values: DataExportFormValues) => {
      const dataExportTargetRequest = toDataExportRequest(values)
      return dataExportApi.putDataExportTarget(
        { orgId, dataExportTargetRequest },
        {
          // The client stringifies a non-string body itself.
          body: toDataExportWireBody(dataExportTargetRequest) as unknown as BodyInit,
        },
      )
    },
    onSuccess: () => showToast(t('organization.dataExport.toast.saved')),
    onError: showFailure,
    onSettled: refresh,
  })

  const remove = useMutation({
    mutationFn: () => dataExportApi.deleteDataExportTarget({ orgId }),
    onSuccess: () => showToast(t('organization.dataExport.toast.removed')),
    onError: showFailure,
    onSettled: refresh,
  })

  const test = useMutation({
    mutationFn: () => dataExportApi.testDataExportTarget({ orgId }),
  })

  return { save, remove, test }
}
