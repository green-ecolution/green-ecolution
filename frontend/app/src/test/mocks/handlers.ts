import { http, HttpResponse } from 'msw'

const baseUrl = '/api-local'

export const handlers = [
  // Tree API handlers
  http.get(`${baseUrl}/v1/tree`, () => {
    return HttpResponse.json({
      data: [],
      pagination: { page: 1, limit: 10, total: 0 },
    })
  }),

  http.post(`${baseUrl}/v1/tree`, async ({ request }) => {
    const body = (await request.json()) as Record<string, unknown>
    return HttpResponse.json(
      {
        id: 1,
        ...body,
        createdAt: new Date().toISOString(),
        updatedAt: new Date().toISOString(),
      },
      { status: 201 },
    )
  }),

  // Sensor API handlers
  http.get(`${baseUrl}/v1/sensor`, () => {
    return HttpResponse.json({
      data: [
        { id: 'sensor-1', status: 'online' },
        { id: 'sensor-2', status: 'offline' },
      ],
      pagination: { page: 1, limit: 10, total: 2 },
    })
  }),

  // TreeCluster API handlers
  http.get(`${baseUrl}/v1/cluster`, () => {
    return HttpResponse.json({
      data: [
        { id: 1, name: 'Cluster A' },
        { id: 2, name: 'Cluster B' },
      ],
      pagination: { page: 1, limit: 10, total: 2 },
    })
  }),
]
