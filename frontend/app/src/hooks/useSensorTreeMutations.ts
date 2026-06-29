import { sensorApi } from '@/api/backendApi'
import { useMutation, useQueryClient } from '@tanstack/react-query'

const invalidateSensorAndTrees = async (
  queryClient: ReturnType<typeof useQueryClient>,
  sensorId: string,
  treeIds: (string | null | undefined)[],
) => {
  await Promise.all([
    queryClient.invalidateQueries({ queryKey: ['sensors'] }),
    queryClient.invalidateQueries({ queryKey: ['sensor', sensorId] }),
    ...treeIds
      .filter((id): id is string => !!id)
      .map((id) => queryClient.invalidateQueries({ queryKey: ['tree', id] })),
  ])
}

export const useActivateSensor = (sensorId: string) => {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: (treeId: string) =>
      sensorApi.activateSensor({ sensorId, activateSensorRequest: { treeId } }),
    onSuccess: (_data, treeId) => invalidateSensorAndTrees(queryClient, sensorId, [treeId]),
  })
}

export const useReassignSensorTree = (sensorId: string, previousTreeId?: string | null) => {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: (treeId: string) =>
      sensorApi.setSensorTree({ sensorId, setSensorTreeRequest: { treeId } }),
    onSuccess: (_data, treeId) =>
      invalidateSensorAndTrees(queryClient, sensorId, [previousTreeId, treeId]),
  })
}

export const useDeactivateSensor = (sensorId: string, previousTreeId?: string | null) => {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: () => sensorApi.removeSensorTree({ sensorId }),
    onSuccess: () => invalidateSensorAndTrees(queryClient, sensorId, [previousTreeId]),
  })
}
