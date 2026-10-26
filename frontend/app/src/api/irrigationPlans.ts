import type {
  ListResponsePlanInListResponse,
  PaginationResponse,
  PlanCreateRequest,
  PlanInListResponse,
  PlanResponse,
  PlanUpdateRequest,
  RouteResponse,
  StationVisitResponse,
  TreeClusterInListResponse,
  StopOutcomeDto,
} from '@green-ecolution/backend-client'
import { PlanStatus } from '@green-ecolution/backend-client'
import { IRRIGATION } from '@/lib/targets'
import { clusterApi } from '../../../modules/irrigation/src/api/client'

// The plan API is module-neutral while the irrigation screens still speak in
// clusters and water; this adapter bridges both until the module package replaces it.

export interface EvaluationValue {
  treeClusterId: string
  wateringPlanId: string
  consumedWater: number
}

type Generic = 'module' | 'stops' | 'outcomes' | 'transporterCapacityUsed' | 'totalDemand'

// `treeclusters` holds only the stops the reader may see; writes must use `stopIds`
// so a save never drops a group outside the reader's visibility.
export type WateringPlan = Omit<PlanResponse, Generic> & {
  stopIds: string[]
  treeclusters: TreeClusterInListResponse[]
  evaluation: EvaluationValue[]
  transporterTankUsed: boolean
  totalWaterRequired: number
}

export type WateringPlanInList = Omit<PlanInListResponse, Generic> & {
  stopIds: string[]
  treeclusters: TreeClusterInListResponse[]
  transporterTankUsed: boolean
  totalWaterRequired: number
}

export interface WateringPlanList {
  data: WateringPlanInList[]
  pagination: PaginationResponse
}

export type WateringPlanCreate = Omit<
  PlanCreateRequest,
  'module' | 'stops' | 'transporterCapacityUsed'
> & {
  treeClusterIds: string[]
  transporterTankUsed?: boolean
}

export type WateringPlanUpdate = Omit<
  PlanUpdateRequest,
  'stops' | 'transporterCapacityUsed' | 'outcomes'
> & {
  treeClusterIds: string[]
  transporterTankUsed?: boolean
  evaluation?: EvaluationValue[]
}

export type WateringRoute = Omit<RouteResponse, 'stationVisits' | 'totalDemand'> & {
  refillPoints: StationVisitResponse[]
  totalWaterRequired: number
}

const PAGE = 100

export async function loadClusters(ids: string[]): Promise<Map<string, TreeClusterInListResponse>> {
  const unique = [...new Set(ids)]
  const map = new Map<string, TreeClusterInListResponse>()
  for (let i = 0; i < unique.length; i += PAGE) {
    const page = await clusterApi.listClusters({ id: unique.slice(i, i + PAGE), perPage: PAGE })
    page.data.forEach((c) => map.set(c.id, c))
  }
  return map
}

const clustersFor = (stops: string[], clusters: Map<string, TreeClusterInListResponse>) =>
  stops.flatMap((id) => {
    const cluster = clusters.get(id)
    return cluster ? [cluster] : []
  })

export const toWateringPlan = (
  plan: PlanResponse,
  clusters: Map<string, TreeClusterInListResponse>,
): WateringPlan => {
  const { module: _module, stops, outcomes, transporterCapacityUsed, totalDemand, ...rest } = plan
  return {
    ...rest,
    stopIds: stops,
    treeclusters: clustersFor(stops, clusters),
    evaluation: outcomes
      .filter((o) => o.completed)
      .map((o) => ({
        treeClusterId: o.stop,
        wateringPlanId: plan.id,
        consumedWater: o.amount ?? 0,
      })),
    transporterTankUsed: transporterCapacityUsed,
    totalWaterRequired: totalDemand,
  }
}

export const toWateringPlanInList = (
  plan: PlanInListResponse,
  clusters: Map<string, TreeClusterInListResponse>,
): WateringPlanInList => {
  const { module: _module, stops, transporterCapacityUsed, totalDemand, ...rest } = plan
  return {
    ...rest,
    stopIds: stops,
    treeclusters: clustersFor(stops, clusters),
    transporterTankUsed: transporterCapacityUsed,
    totalWaterRequired: totalDemand,
  }
}

export const keepUnresolvedStops = (
  selected: string[],
  plan: Pick<WateringPlan, 'stopIds' | 'treeclusters'>,
): string[] => {
  const resolved = new Set(plan.treeclusters.map((cluster) => cluster.id))
  const unresolved = plan.stopIds.filter((id) => !resolved.has(id) && !selected.includes(id))
  return [...selected, ...unresolved]
}

export async function loadWateringPlanList(
  page: ListResponsePlanInListResponse,
): Promise<WateringPlanList> {
  const clusters = await loadClusters(page.data.flatMap((plan) => plan.stops))
  return {
    pagination: page.pagination,
    data: page.data.map((plan) => toWateringPlanInList(plan, clusters)),
  }
}

export const toPlanCreateRequest = (form: WateringPlanCreate): PlanCreateRequest => {
  const { treeClusterIds, transporterTankUsed, ...rest } = form
  return {
    ...rest,
    module: IRRIGATION,
    stops: treeClusterIds,
    transporterCapacityUsed: transporterTankUsed ?? true,
  }
}

// A plan is finished as a whole: stops without an evaluation (groups the reader
// cannot see) count as done with an unknown amount.
const toOutcomes = (
  status: PlanStatus,
  stops: string[],
  evaluation: EvaluationValue[] | undefined,
): StopOutcomeDto[] | undefined => {
  const evaluated: StopOutcomeDto[] | undefined = evaluation?.map((e) => ({
    stop: e.treeClusterId,
    completed: true,
    amount: e.consumedWater,
  }))
  if (status !== PlanStatus.Finished) return evaluated
  const covered = new Set(evaluated?.map((o) => o.stop))
  const rest = stops
    .filter((stop) => !covered.has(stop))
    .map((stop) => ({ stop, completed: true, amount: null }))
  return [...(evaluated ?? []), ...rest]
}

export const toPlanUpdateRequest = (form: WateringPlanUpdate): PlanUpdateRequest => {
  const { treeClusterIds, transporterTankUsed, evaluation, ...rest } = form
  return {
    ...rest,
    stops: treeClusterIds,
    transporterCapacityUsed: transporterTankUsed ?? true,
    outcomes: toOutcomes(form.status, treeClusterIds, evaluation),
  }
}

export const toWateringRoute = (route: RouteResponse): WateringRoute => {
  const { stationVisits, totalDemand, ...rest } = route
  return { ...rest, refillPoints: stationVisits, totalWaterRequired: totalDemand }
}
