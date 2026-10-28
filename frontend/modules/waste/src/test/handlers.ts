import { http, HttpResponse } from 'msw'
import type { WasteBinResponse } from '@green-ecolution/backend-client'

const baseUrl = '/api-local/v1'

export const BIN_FULL_ID = '0190a8e9-7c4f-7000-8000-00000000b001'
export const BIN_NORMAL_ID = '0190a8e9-7c4f-7000-8000-00000000b002'

// Wire format: the generated client maps these snake_case keys to camelCase.
const wireBin = (bin: WasteBinResponse) => ({
  id: bin.id,
  number: bin.number,
  latitude: bin.latitude,
  longitude: bin.longitude,
  volume_liters: bin.volumeLiters,
  depth_cm: bin.depthCm,
  organization_id: bin.organizationId,
  provider: bin.provider ?? null,
  fill_percent: bin.fillPercent ?? null,
  fill_measured_at: bin.fillMeasuredAt ?? null,
  last_emptied_at: bin.lastEmptiedAt ?? null,
  fill_state: bin.fillState,
  sensor_id: bin.sensorId ?? null,
  created_at: bin.createdAt,
  updated_at: bin.updatedAt,
})

export const wasteBinFixtures: WasteBinResponse[] = [
  {
    id: BIN_FULL_ID,
    number: 'B-0042',
    latitude: 54.7836,
    longitude: 9.4321,
    volumeLiters: 1100,
    depthCm: 120,
    organizationId: 'org-1',
    fillPercent: 85,
    fillMeasuredAt: '2026-10-27T08:30:00Z',
    lastEmptiedAt: '2026-10-20T06:00:00Z',
    fillState: 'full',
    sensorId: 'eui-a84041000181c2f1',
    createdAt: '2026-10-01T00:00:00Z',
    updatedAt: '2026-10-27T08:30:00Z',
  },
  {
    id: BIN_NORMAL_ID,
    number: 'B-0043',
    latitude: 54.7801,
    longitude: 9.4402,
    volumeLiters: 240,
    depthCm: 90,
    organizationId: 'org-1',
    fillPercent: 40,
    fillMeasuredAt: '2026-10-27T07:00:00Z',
    lastEmptiedAt: null,
    fillState: 'normal',
    sensorId: null,
    createdAt: '2026-10-02T00:00:00Z',
    updatedAt: '2026-10-27T07:00:00Z',
  },
]

const page = (data: WasteBinResponse[]) => ({
  data: data.map(wireBin),
  pagination: {
    current_page: 1,
    next_page: null,
    per_page: 25,
    prev_page: null,
    total_pages: 1,
    total_records: data.length,
    total_unfiltered: wasteBinFixtures.length,
  },
})

export const wasteHandlers = [
  http.get(`${baseUrl}/waste-bins`, ({ request }) => {
    const url = new URL(request.url)
    const fillState = url.searchParams.get('fill_state')
    const number = url.searchParams.get('number')?.toLowerCase()
    const bins = wasteBinFixtures.filter(
      (bin) =>
        (!fillState || bin.fillState === fillState) &&
        (!number || bin.number.toLowerCase().includes(number)),
    )
    return HttpResponse.json(page(bins))
  }),

  http.get(`${baseUrl}/waste-bins/:id`, ({ params }) => {
    const bin = wasteBinFixtures.find((candidate) => candidate.id === params.id)
    return bin
      ? HttpResponse.json(wireBin(bin))
      : HttpResponse.json({ error: 'not found' }, { status: 404 })
  }),

  http.post(`${baseUrl}/waste-bins`, async ({ request }) => {
    const body = (await request.json()) as Record<string, unknown>
    return HttpResponse.json(
      {
        ...wireBin(wasteBinFixtures[1]),
        ...body,
        id: '0190a8e9-7c4f-7000-8000-00000000b0ff',
        fill_state: 'unknown',
        fill_percent: null,
      },
      { status: 201 },
    )
  }),

  http.put(`${baseUrl}/waste-bins/:id`, async ({ params, request }) => {
    const body = (await request.json()) as Record<string, unknown>
    const bin = wasteBinFixtures.find((candidate) => candidate.id === params.id)
    return bin
      ? HttpResponse.json({ ...wireBin(bin), ...body })
      : HttpResponse.json({ error: 'not found' }, { status: 404 })
  }),

  http.delete(`${baseUrl}/waste-bins/:id`, () => new HttpResponse(null, { status: 204 })),
]
