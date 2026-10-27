import { beforeEach, describe, expect, it, vi } from 'vitest'
import { render, screen, waitFor } from '@testing-library/react'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import type { SensorResponse } from '@green-ecolution/backend-client'

const { abilities } = vi.hoisted(() => ({ abilities: { current: [] as string[] } }))

vi.mock('@/api/queries', async (importOriginal) => {
  const actual = await importOriginal<typeof import('@/api/queries')>()
  return {
    ...actual,
    sensorQueries: {
      ...actual.sensorQueries,
      model: (id: string) => ({
        queryKey: ['chart-test-model', id, abilities.current],
        queryFn: () =>
          Promise.resolve({ id, abilities: abilities.current.map((ability) => ({ ability })) }),
      }),
      dataQuality: (id: string) => ({
        queryKey: ['chart-test-quality', id],
        queryFn: () => Promise.resolve({ issues: [] }),
      }),
    },
  }
})

vi.mock('../../api/queries', () => ({
  sensorSoilMoistureQuery: (id: string) => ({
    queryKey: ['chart-test-series', id],
    queryFn: () => Promise.resolve({ series: [], thresholds: [], wateringEvents: [] }),
  }),
}))

const { default: SensorSoilMoistureChart } = await import('./SensorSoilMoistureChart')

const renderChart = (target: SensorResponse['target']) => {
  const queryClient = new QueryClient()
  const view = render(
    <QueryClientProvider client={queryClient}>
      <SensorSoilMoistureChart
        sensor={{ id: 'eui-1', model: { id: 'model-1' }, target } as SensorResponse}
      />
    </QueryClientProvider>,
  )
  return { ...view, queryClient }
}

describe('SensorSoilMoistureChart', () => {
  beforeEach(() => {
    abilities.current = ['soil_moisture']
  })

  it('shows the series of a soil moisture sensor that is linked to nothing', async () => {
    renderChart(null)

    expect(await screen.findByText('Bodenfeuchte-Verlauf')).toBeInTheDocument()
  })

  it('shows the series of a soil moisture sensor on a tree', async () => {
    renderChart({ module: 'irrigation', kind: 'tree', id: 'tree-1' })

    expect(await screen.findByText('Bodenfeuchte-Verlauf')).toBeInTheDocument()
  })

  it('stays away for a sensor whose model does not measure soil moisture', async () => {
    abilities.current = ['temperature']
    const { container, queryClient } = renderChart({
      module: 'irrigation',
      kind: 'tree',
      id: 'tree-1',
    })

    await waitFor(() =>
      expect(
        queryClient.getQueryData(['chart-test-model', 'model-1', ['temperature']]),
      ).toBeDefined(),
    )
    expect(container).toBeEmptyDOMElement()
  })
})
