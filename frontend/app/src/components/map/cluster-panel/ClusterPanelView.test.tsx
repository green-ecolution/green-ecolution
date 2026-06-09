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
  moistureLevel: 14,
  wateringStatus: 'bad',
  lastWatered: null,
  region: { name: 'Flensburg' },
  trees: [
    { id: 't1', species: 'Spitzahorn', number: '10429', sensor: null },
    { id: 't2', species: 'Stieleiche', number: '10428', sensor: { id: 's1' } },
  ],
} as unknown as TreeClusterResponse

afterEach(cleanup)

describe('ClusterPanelView', () => {
  it('renders name, address and tree count', () => {
    render(<ClusterPanelView treecluster={cluster} onEdit={vi.fn()} onOpenDashboard={vi.fn()} />)
    expect(screen.getByRole('heading', { name: 'Hafenspitze' })).toBeInTheDocument()
    expect(screen.getByText(/Schiffbrücke 12/)).toBeInTheDocument()
  })

  it('lists sensor trees first', () => {
    render(<ClusterPanelView treecluster={cluster} onEdit={vi.fn()} onOpenDashboard={vi.fn()} />)
    const items = screen.getAllByTestId('cluster-panel-tree')
    expect(items[0]).toHaveTextContent('Stieleiche')
    expect(items[0]).toHaveTextContent('Sensor-Baum')
  })

  it('shows the moisture level', () => {
    render(<ClusterPanelView treecluster={cluster} onEdit={vi.fn()} onOpenDashboard={vi.fn()} />)
    expect(screen.getByText('14 %')).toBeInTheDocument()
  })

  it('fires onEdit and onOpenDashboard', async () => {
    const onEdit = vi.fn()
    const onOpenDashboard = vi.fn()
    render(<ClusterPanelView treecluster={cluster} onEdit={onEdit} onOpenDashboard={onOpenDashboard} />)
    await userEvent.click(screen.getByRole('button', { name: 'Gruppe bearbeiten' }))
    await userEvent.click(screen.getByRole('button', { name: 'Zum Dashboard' }))
    expect(onEdit).toHaveBeenCalledTimes(1)
    expect(onOpenDashboard).toHaveBeenCalledTimes(1)
  })
})
