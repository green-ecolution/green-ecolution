import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { render, screen, cleanup, userEvent, waitFor } from '@/test/utils'

const getCluster = vi.fn()
vi.mock('@/api/backendApi', async (importOriginal) => {
  const actual = await importOriginal<typeof import('@/api/backendApi')>()
  return { ...actual, clusterApi: { getCluster: (...a: unknown[]) => getCluster(...a) as unknown } }
})
vi.mock('@/hooks/useMediaQuery', () => ({ useMediaQuery: () => true }))

import ClusterPanel from './ClusterPanel'

const VALID_ID = '11111111-1111-4111-8111-111111111111'
const cluster = {
  id: VALID_ID,
  name: 'Hafenspitze',
  address: 'Schiffbrücke 12',
  description: '',
  soilCondition: 'sandig',
  moistureLevel: 14,
  wateringStatus: 'bad',
  lastWatered: null,
  trees: [],
}

beforeEach(() => vi.clearAllMocks())
afterEach(cleanup)

describe('ClusterPanel', () => {
  it('shows the view for a loaded cluster and calls onEdit from the pencil', async () => {
    getCluster.mockResolvedValue(cluster)
    const onEdit = vi.fn()
    render(
      <ClusterPanel
        clusterId={VALID_ID}
        onClose={vi.fn()}
        onOpenDashboard={vi.fn()}
        onEdit={onEdit}
      />,
    )

    await waitFor(() =>
      expect(screen.getByRole('heading', { name: 'Hafenspitze' })).toBeInTheDocument(),
    )
    await userEvent.click(screen.getByRole('button', { name: 'Gruppe bearbeiten' }))
    expect(onEdit).toHaveBeenCalledTimes(1)
  })

  it('shows an error state for an invalid (non-uuid) id without hanging on the spinner', () => {
    render(
      <ClusterPanel
        clusterId="not-a-uuid"
        onClose={vi.fn()}
        onOpenDashboard={vi.fn()}
        onEdit={vi.fn()}
      />,
    )
    expect(screen.getByText(/konnte nicht geladen werden/)).toBeInTheDocument()
    expect(screen.queryByText(/Lade Baumgruppe/)).not.toBeInTheDocument()
    expect(getCluster).not.toHaveBeenCalled()
  })

  it('calls onClose from the close button', async () => {
    getCluster.mockResolvedValue(cluster)
    const onClose = vi.fn()
    render(
      <ClusterPanel
        clusterId={VALID_ID}
        onClose={onClose}
        onOpenDashboard={vi.fn()}
        onEdit={vi.fn()}
      />,
    )
    await waitFor(() =>
      expect(screen.getByRole('heading', { name: 'Hafenspitze' })).toBeInTheDocument(),
    )
    await userEvent.click(screen.getByRole('button', { name: 'Seitenansicht schließen' }))
    expect(onClose).toHaveBeenCalledTimes(1)
  })
})
