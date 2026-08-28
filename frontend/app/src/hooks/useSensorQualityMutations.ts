import { useMutation, useQueryClient } from '@tanstack/react-query'
import { sensorApi } from '@/api/backendApi'
import createToast from '@/hooks/createToast'

export interface AcknowledgeDataQualityVariables {
  sensorId: string
  note?: string
}

export const useSensorQualityMutations = () => {
  const queryClient = useQueryClient()
  const showToast = createToast()

  const acknowledge = useMutation({
    mutationFn: ({ sensorId, note }: AcknowledgeDataQualityVariables) => {
      // A blank note must not travel: the backend rejects it as invalid input.
      const trimmed = note?.trim()
      return sensorApi.acknowledgeSensorDataQuality({
        sensorId,
        acknowledgeDataQualityRequest: { note: trimmed === '' ? undefined : trimmed },
      })
    },
    onSuccess: (_data, { sensorId }) => {
      // The badge on the sensor list and the notices on tree and cluster cards
      // read the same two fields, so the sensor caches have to go too.
      void queryClient.invalidateQueries({ queryKey: ['sensor', sensorId] })
      void queryClient.invalidateQueries({ queryKey: ['sensors'] })
      showToast('Datenqualität als geprüft vermerkt')
    },
    onError: () => {
      showToast('Der Vermerk konnte nicht gespeichert werden.', 'error')
    },
  })

  return { acknowledge }
}
