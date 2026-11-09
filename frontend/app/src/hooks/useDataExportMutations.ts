import { useMutation, useQueryClient } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'
import { dataExportApi } from '@/api/backendApi'
import { dataExportQueries, toDataExportRequest, type DataExportFormValues } from '@/api/dataExport'
import createToast from '@/hooks/createToast'
import { resolveApiError } from '@/lib/apiError'

export const useDataExportMutations = (orgId: string) => {
  const queryClient = useQueryClient()
  const showToast = createToast()
  const { t } = useTranslation('settings')

  const refresh = () =>
    queryClient.invalidateQueries({ queryKey: dataExportQueries.byOrganization(orgId).queryKey })

  const showFailure = (error: unknown) => {
    void resolveApiError(error).then((info) => showToast(info.message, 'error'))
  }

  const save = useMutation({
    mutationFn: (values: DataExportFormValues) =>
      dataExportApi.putDataExportTarget({
        orgId,
        dataExportTargetRequest: toDataExportRequest(values),
      }),
    // No error toast: the dialog owns a failed save and shows it in place.
    onSuccess: () => showToast(t('organization.dataExport.toast.saved')),
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
