import { useMutation } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'
import type { StartPointRequest } from '@/api/backendApi'
import { routingApi } from '@/api/backendApi'
import { queryRoots } from '@/api/queries'
import createToast from '@/hooks/createToast'
import { resolveApiError } from '@/lib/apiError'
import { useInvalidateQueryRoots } from '@/lib/queryInvalidation'

export interface UpdateStationVariables {
  id: string
  body: StartPointRequest
}

export const useStationMutations = () => {
  const invalidate = useInvalidateQueryRoots()
  const showToast = createToast()
  const { t } = useTranslation('settings')

  const refresh = () => invalidate(queryRoots.startPoint)

  // Refusals such as deleting the default station carry their own explanation.
  const showFailure = (error: unknown) => {
    void resolveApiError(error).then((info) =>
      showToast(info.message || t('stations.toast.failed'), 'error'),
    )
  }

  const create = useMutation({
    mutationFn: (startPointRequest: StartPointRequest) =>
      routingApi.createStartPoint({ startPointRequest }),
    onSuccess: () => {
      void refresh()
      showToast(t('stations.toast.created'))
    },
    onError: showFailure,
  })

  const update = useMutation({
    mutationFn: ({ id, body }: UpdateStationVariables) =>
      routingApi.updateStartPoint({ startPointId: id, startPointRequest: body }),
    onSuccess: () => {
      void refresh()
      showToast(t('stations.toast.updated'))
    },
    onError: showFailure,
  })

  const remove = useMutation({
    mutationFn: (id: string) => routingApi.deleteStartPoint({ startPointId: id }),
    onSuccess: () => {
      void refresh()
      showToast(t('stations.toast.deleted'))
    },
    onError: showFailure,
  })

  const setDefault = useMutation({
    mutationFn: (id: string) => routingApi.setDefaultStartPoint({ startPointId: id }),
    onSuccess: () => {
      void refresh()
      showToast(t('stations.toast.defaultSet'))
    },
    onError: showFailure,
  })

  return { create, update, remove, setDefault }
}
