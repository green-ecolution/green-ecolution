import { z } from 'zod'
import { sensorListFiltersOf } from '@/lib/targets'
import type { ModuleManifest } from '@/modules/types'
import { moduleManifests } from '@/modules/registry'
import {
  DataHealth,
  ListSensorsOrderEnum,
  type ListSensorsRequest,
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
  attached: z.boolean().optional().catch(undefined),
  /** Read only to redirect links from before `attached`. */
  hasTree: z.boolean().optional().catch(undefined),
  sort: z.enum(ListSensorsSortEnum).optional().catch(undefined),
  order: z.enum(ListSensorsOrderEnum).optional().catch(undefined),
})

/** Module filter params stay loose here; each module reads its own through `listFilterValue`. */
export type SensorListSearch = z.output<typeof sensorFilterSchema> & Record<string, unknown>

export function sensorSearchParserFor(
  manifests: readonly ModuleManifest[],
): (search: Record<string, unknown>) => SensorListSearch {
  const moduleSchema = z.object(
    Object.fromEntries(
      sensorListFiltersOf(manifests).map(({ filter }) => [
        filter.param,
        z.array(z.string()).optional().catch(undefined),
      ]),
    ),
  )
  return (search) => ({ ...moduleSchema.parse(search), ...sensorFilterSchema.parse(search) })
}

/** Every registered module's filter param, active or not: a shared link keeps them before the active modules are known. */
export const MODULE_FILTER_PARAMS = sensorListFiltersOf(moduleManifests).map(
  ({ filter }) => filter.param,
)

export const ACTIVATED_STATUSES: SensorStatus[] = [SensorStatus.Online, SensorStatus.Offline]

// Prepared sensors have no readings and no target, so these filters could
// only ever empty the prepared view.
export const ACTIVATED_ONLY_FILTERS: Partial<SensorListSearch> = {
  statuses: undefined,
  dataHealth: undefined,
  attached: undefined,
  ...Object.fromEntries(MODULE_FILTER_PARAMS.map((param) => [param, undefined])),
}

export const SENSOR_LIST_PER_PAGE = 25

export const sensorListParams = (
  search: SensorListSearch,
  filteredSensorIds?: string[],
): ListSensorsRequest => ({
  page: search.page,
  perPage: SENSOR_LIST_PER_PAGE,
  q: search.q,
  status: statusesForView(search.view ?? 'activated', search.statuses),
  modelId: search.modelIds,
  dataHealth: search.dataHealth,
  attached: search.attached,
  id: filteredSensorIds,
  sort: search.sort,
  order: search.order,
})

export const statusesForView = (
  view: SensorListView,
  statuses: SensorStatus[] | undefined,
): SensorStatus[] => {
  if (view === 'prepared') return [SensorStatus.Prepared]
  const picked = (statuses ?? []).filter((status) => ACTIVATED_STATUSES.includes(status))
  return picked.length > 0 ? picked : ACTIVATED_STATUSES
}

/** Links from before the link filter turned target-neutral carry `hasTree`. */
export const legacyHasTreeRedirect = (search: SensorListSearch): SensorListSearch | undefined => {
  if (search.hasTree === undefined) return undefined
  return { ...search, hasTree: undefined, attached: search.attached ?? search.hasTree }
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
