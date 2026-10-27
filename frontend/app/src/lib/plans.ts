import type {
  PlanCreateRequest,
  PlanInListResponse,
  PlanUpdateRequest,
} from '@green-ecolution/backend-client'
import type { WateringPlanForm } from '@/schema/wateringPlanSchema'

const trailerOf = (trailerId: string | undefined) =>
  trailerId && trailerId !== '-1' ? trailerId : undefined

export const toPlanCreateRequest = (form: WateringPlanForm): PlanCreateRequest => ({
  module: form.module,
  date: form.date.toISOString(),
  description: form.description,
  transporterId: form.transporterId,
  trailerId: trailerOf(form.trailerId),
  transporterCapacityUsed: form.transporterTankUsed ?? true,
  stops: form.stopIds,
  userIds: form.driverIds,
  startPointName: form.startPointName,
})

export const toPlanFormUpdateRequest = (form: WateringPlanForm): PlanUpdateRequest => ({
  date: form.date.toISOString(),
  description: form.description,
  status: form.status,
  transporterId: form.transporterId,
  trailerId: trailerOf(form.trailerId),
  transporterCapacityUsed: form.transporterTankUsed ?? true,
  stops: form.stopIds,
  userIds: form.driverIds,
  startPointName: form.startPointName,
  cancellationNote: '',
})

/** Writes a plan back as it is, changed only by `overrides`; used for status changes and assignments. */
export const toUpdateRequest = (
  plan: PlanInListResponse & { startPointName?: string | null },
  overrides: Partial<PlanUpdateRequest>,
): PlanUpdateRequest => ({
  date: plan.date,
  description: plan.description,
  status: plan.status,
  transporterId: plan.transporter.id,
  trailerId: plan.trailer?.id,
  transporterCapacityUsed: plan.transporterCapacityUsed,
  stops: plan.stops,
  userIds: plan.userIds,
  startPointName: plan.startPointName,
  cancellationNote: plan.cancellationNote,
  provider: plan.provider,
  additionalInformation: plan.additionalInformation,
  ...overrides,
})
