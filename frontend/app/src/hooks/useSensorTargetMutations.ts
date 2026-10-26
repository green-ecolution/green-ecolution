import type { TargetRefDto } from '@green-ecolution/backend-client'
import { useMutation, type QueryKey } from '@tanstack/react-query'
import { queryRoots } from '@/api/queries'
import { sensorApi } from '@/api/backendApi'
import { useInvalidateQueryRoots } from '@/lib/queryInvalidation'

/** Key prefixes of the modules whose objects gain or lose the sensor. */
interface LinkChange {
  invalidate: readonly QueryKey[]
}

/**
 * The sensor detail page stays mounted afterwards and its breadcrumb comes
 * from loader data, so the route loaders re-run too.
 */
const useInvalidateSensorLink = () => {
  const invalidate = useInvalidateQueryRoots()
  return (_: unknown, { invalidate: moduleRoots }: LinkChange) =>
    invalidate([...queryRoots.sensor, ...moduleRoots], { reloadRoutes: true })
}

export const useActivateSensor = (sensorId: string) => {
  const onSuccess = useInvalidateSensorLink()
  return useMutation({
    mutationFn: ({ target }: LinkChange & { target: TargetRefDto }) =>
      sensorApi.activateSensor({ sensorId, activateSensorRequest: { target } }),
    onSuccess,
  })
}

export const useReassignSensor = (sensorId: string) => {
  const onSuccess = useInvalidateSensorLink()
  return useMutation({
    mutationFn: ({ target }: LinkChange & { target: TargetRefDto }) =>
      sensorApi.setSensorTarget({ sensorId, setSensorTargetRequest: { target } }),
    onSuccess,
  })
}

export const useDeactivateSensor = (sensorId: string) => {
  const onSuccess = useInvalidateSensorLink()
  return useMutation({
    mutationFn: (_: LinkChange) => sensorApi.removeSensorTarget({ sensorId }),
    onSuccess,
  })
}
