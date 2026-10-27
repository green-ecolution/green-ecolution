import { PlanStatus } from '@green-ecolution/backend-client'
import type {
  PlanInListResponse,
  PlanResponse,
  VehicleResponse,
} from '@green-ecolution/backend-client'

export function planInList(overrides: Partial<PlanInListResponse> = {}): PlanInListResponse {
  return {
    id: '0190a8e9-7c4f-7000-8000-000000000000',
    module: 'irrigation',
    createdAt: '2026-07-01T00:00:00Z',
    updatedAt: '2026-07-01T00:00:00Z',
    date: '2026-07-08T00:00:00Z',
    description: 'Innenstadt',
    status: PlanStatus.Planned,
    distance: 1000,
    totalDemand: 4200,
    cancellationNote: '',
    transporter: { id: 'v-1', numberPlate: 'FL-GE-A01' } as VehicleResponse,
    transporterCapacityUsed: true,
    stops: [],
    userIds: [],
    ...overrides,
  }
}

export function planResponse(overrides: Partial<PlanResponse> = {}): PlanResponse {
  return {
    ...planInList(),
    duration: 0,
    gpxUrl: '/plans/route.gpx',
    organizationId: '0190a8e9-7c4f-7000-8000-0000000000aa',
    outcomes: [],
    refillCount: 0,
    ...overrides,
  }
}
