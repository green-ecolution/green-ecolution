import { queryRoots } from '@/api/queries'
import { irrigationRoots } from '../../../modules/irrigation/src/api/queries'
import { sensorApi } from '@/api/backendApi'
import { useMutation } from '@tanstack/react-query'
import { useInvalidateQueryRoots } from '@/lib/queryInvalidation'
import { treeTarget } from '@/lib/targets'

/**
 * Sensor/tree link changes. The sensor detail page stays mounted afterwards and
 * its breadcrumb comes from loader data, so the route loaders re-run too.
 */
const useInvalidateSensorTreeLink = () => {
  const invalidate = useInvalidateQueryRoots()
  return () => invalidate([...queryRoots.sensor, ...irrigationRoots.tree], { reloadRoutes: true })
}

export const useActivateSensor = (sensorId: string) => {
  const invalidateLink = useInvalidateSensorTreeLink()
  return useMutation({
    mutationFn: (treeId: string) =>
      sensorApi.activateSensor({ sensorId, activateSensorRequest: { target: treeTarget(treeId) } }),
    onSuccess: invalidateLink,
  })
}

export const useReassignSensorTree = (sensorId: string) => {
  const invalidateLink = useInvalidateSensorTreeLink()
  return useMutation({
    mutationFn: (treeId: string) =>
      sensorApi.setSensorTarget({
        sensorId,
        setSensorTargetRequest: { target: treeTarget(treeId) },
      }),
    onSuccess: invalidateLink,
  })
}

export const useDeactivateSensor = (sensorId: string) => {
  const invalidateLink = useInvalidateSensorTreeLink()
  return useMutation({
    mutationFn: () => sensorApi.removeSensorTarget({ sensorId }),
    onSuccess: invalidateLink,
  })
}
