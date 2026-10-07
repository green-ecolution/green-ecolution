import { z } from 'zod'
import {
  DataHealth,
  ListSensorsOrderEnum,
  ListSensorsSortEnum,
  SensorStatus,
} from '@green-ecolution/backend-client'

export const SENSOR_LIST_VIEWS = ['activated', 'prepared'] as const
export type SensorListView = (typeof SENSOR_LIST_VIEWS)[number]

export const sensorFilterSchema = z.object({
  page: z.number().int().min(1).catch(1),
  view: z.enum(SENSOR_LIST_VIEWS).optional().catch(undefined),
  q: z.string().optional().catch(undefined),
  statuses: z.array(z.enum(SensorStatus)).optional().catch(undefined),
  modelIds: z.array(z.string()).optional().catch(undefined),
  dataHealth: z.array(z.enum(DataHealth)).optional().catch(undefined),
  hasTree: z.boolean().optional().catch(undefined),
  clusterIds: z.array(z.string()).optional().catch(undefined),
  sort: z.enum(ListSensorsSortEnum).optional().catch(undefined),
  order: z.enum(ListSensorsOrderEnum).optional().catch(undefined),
})

export type SensorListSearch = z.infer<typeof sensorFilterSchema>

export const ACTIVATED_STATUSES: SensorStatus[] = [SensorStatus.Online, SensorStatus.Offline]

// Prepared sensors have no readings, tree or cluster, so these filters could
// only ever empty the prepared view.
export const ACTIVATED_ONLY_FILTERS = {
  statuses: undefined,
  dataHealth: undefined,
  hasTree: undefined,
  clusterIds: undefined,
} as const

export const statusesForView = (
  view: SensorListView,
  statuses: SensorStatus[] | undefined,
): SensorStatus[] => {
  if (view === 'prepared') return [SensorStatus.Prepared]
  const picked = (statuses ?? []).filter((status) => ACTIVATED_STATUSES.includes(status))
  return picked.length > 0 ? picked : ACTIVATED_STATUSES
}

/**
 * Links from before the prepared tab filtered with `statuses=prepared`. Returns
 * the search to redirect to, or undefined when the search is already current.
 */
export const legacyPreparedRedirect = (search: SensorListSearch): SensorListSearch | undefined => {
  if (search.view === 'prepared' || !search.statuses?.includes(SensorStatus.Prepared)) {
    return undefined
  }
  const remaining = search.statuses.filter((status) => status !== SensorStatus.Prepared)
  if (remaining.length > 0) return { ...search, statuses: remaining }
  return { ...search, ...ACTIVATED_ONLY_FILTERS, view: 'prepared', page: 1 }
}
