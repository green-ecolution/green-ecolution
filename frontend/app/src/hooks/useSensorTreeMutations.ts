import { sensorApi } from '@/api/backendApi'
import { useMutation } from '@tanstack/react-query'
import { useInvalidateAggregates } from '@/lib/queryInvalidation'

/**
 * Sensor/tree link changes. The sensor detail page stays mounted afterwards and
 * its breadcrumb comes from loader data, so the route loaders re-run too.
 */
const useInvalidateSensorTreeLink = () => {
  const invalidate = useInvalidateAggregates()
  return () => invalidate(['sensor', 'tree'], { reloadRoutes: true })
}

export const useActivateSensor = (sensorId: string) => {
  const invalidateLink = useInvalidateSensorTreeLink()
  return useMutation({
    mutationFn: (treeId: string) =>
      sensorApi.activateSensor({ sensorId, activateSensorRequest: { treeId } }),
    onSuccess: invalidateLink,
  })
}

export const useReassignSensorTree = (sensorId: string) => {
  const invalidateLink = useInvalidateSensorTreeLink()
  return useMutation({
    mutationFn: (treeId: string) =>
      sensorApi.setSensorTree({ sensorId, setSensorTreeRequest: { treeId } }),
    onSuccess: invalidateLink,
  })
}

export const useDeactivateSensor = (sensorId: string) => {
  const invalidateLink = useInvalidateSensorTreeLink()
  return useMutation({
    mutationFn: () => sensorApi.removeSensorTree({ sensorId }),
    onSuccess: invalidateLink,
  })
}
