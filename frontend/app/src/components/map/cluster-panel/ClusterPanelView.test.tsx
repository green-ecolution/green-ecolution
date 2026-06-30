import { describe, it, expect, vi, afterEach } from 'vitest'
import { render, screen, cleanup, userEvent } from '@/test/utils'
import type { TreeClusterResponse } from '@/api/backendApi'
import ClusterPanelView from './ClusterPanelView'

const cluster = {
  id: 'c1',
  name: 'Hafenspitze',
  address: 'Schiffbrücke 12',
  description: '',
  soilCondition: 'sandig',
  moistureLevel: 0.14,
  wateringStatus: 'bad',
  lastWatered: null,
  region: { name: 'Flensburg' },
  trees: [
    { id: 't1', species: 'Spitzahorn', number: '10429', sensor: null },
    {
      id: 't2',
      species: 'Stieleiche',
      number: '10428',
      sensor: {
        id: 's1',
        latestData: { createdAt: '2026-06-09T08:00:00Z', data: { temperature: 21.4 } },
      },
    },
  ],
} as unknown as TreeClusterResponse

afterEach(cleanup)

describe('ClusterPanelView', () => {
  it('renders address and tree count', () => {
    render(<ClusterPanelView treecluster={cluster} onOpenDashboard={vi.fn()} />)
    expect(screen.getByText(/Schiffbrücke 12/)).toBeInTheDocument()
    expect(screen.getByText(/2 Bäume/)).toBeInTheDocument()
  })

  it('lists sensor trees first', () => {
    render(<ClusterPanelView treecluster={cluster} onOpenDashboard={vi.fn()} />)
    const items = screen.getAllByTestId('cluster-panel-tree')
    expect(items[0]).toHaveTextContent('Stieleiche')
    expect(items[0]).toHaveTextContent('Sensor-Baum')
  })

  it('shows the moisture level scaled to a percentage', () => {
    render(<ClusterPanelView treecluster={cluster} onOpenDashboard={vi.fn()} />)
    expect(screen.getByText('14 %')).toBeInTheDocument()
  })

  it('shows the soil temperature from the latest sensor reading', () => {
    render(<ClusterPanelView treecluster={cluster} onOpenDashboard={vi.fn()} />)
    expect(screen.getByText('21.4 °C')).toBeInTheDocument()
  })

  it('fires onOpenDashboard', async () => {
    const onOpenDashboard = vi.fn()
    render(<ClusterPanelView treecluster={cluster} onOpenDashboard={onOpenDashboard} />)
    await userEvent.click(screen.getByRole('button', { name: 'Zum Dashboard' }))
    expect(onOpenDashboard).toHaveBeenCalledTimes(1)
  })
})
