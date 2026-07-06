import { sensorApi } from '@/api/backendApi'
import { sensorIdQuery, sensorsKey, treeIdQuery } from '@/api/queries'
import { useMutation, useQueryClient } from '@tanstack/react-query'
import { useRouter } from '@tanstack/react-router'
import type { RegisteredRouter } from '@tanstack/react-router'

const invalidateSensorAndTrees = async (
  queryClient: ReturnType<typeof useQueryClient>,
  router: RegisteredRouter,
  sensorId: string,
  treeIds: (string | null | undefined)[],
) => {
  await Promise.all([
    queryClient.invalidateQueries({ queryKey: sensorsKey }),
    queryClient.invalidateQueries({ queryKey: sensorIdQuery(sensorId).queryKey }),
    ...treeIds
      .filter((id): id is string => !!id)
      .map((id) => queryClient.invalidateQueries({ queryKey: treeIdQuery(id).queryKey })),
  ])
  // Detail routes read the sensor/tree from loader data; re-run their loaders.
  await router.invalidate()
}

export const useActivateSensor = (sensorId: string) => {
  const queryClient = useQueryClient()
  const router = useRouter()
  return useMutation({
    mutationFn: (treeId: string) =>
      sensorApi.activateSensor({ sensorId, activateSensorRequest: { treeId } }),
    onSuccess: (_data, treeId) => invalidateSensorAndTrees(queryClient, router, sensorId, [treeId]),
  })
}

export const useReassignSensorTree = (sensorId: string, previousTreeId?: string | null) => {
  const queryClient = useQueryClient()
  const router = useRouter()
  return useMutation({
    mutationFn: (treeId: string) =>
      sensorApi.setSensorTree({ sensorId, setSensorTreeRequest: { treeId } }),
    onSuccess: (_data, treeId) =>
      invalidateSensorAndTrees(queryClient, router, sensorId, [previousTreeId, treeId]),
  })
}

export const useDeactivateSensor = (sensorId: string, previousTreeId?: string | null) => {
  const queryClient = useQueryClient()
  const router = useRouter()
  return useMutation({
    mutationFn: () => sensorApi.removeSensorTree({ sensorId }),
    onSuccess: () => invalidateSensorAndTrees(queryClient, router, sensorId, [previousTreeId]),
  })
}
