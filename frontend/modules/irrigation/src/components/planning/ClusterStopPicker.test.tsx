import { describe, it, expect, vi, beforeEach } from 'vitest'
import userEvent from '@testing-library/user-event'
import type { TreeClusterInListResponse } from '@green-ecolution/backend-client'
import { render, screen } from '@/test/utils'
import type { StopPickerProps } from '@/modules/types'
import ClusterStopPicker from './ClusterStopPicker'

const navigate = vi.fn().mockResolvedValue(undefined)
vi.mock('@tanstack/react-router', () => ({ useNavigate: () => navigate }))
vi.mock('@/components/map-gl/useMapView', () => ({
  useMapCenter: () => [54.78, 9.43],
  useMapZoom: () => 14,
}))
vi.mock('../general/cards/SelectedCard', () => ({
  default: ({ id, onClick }: { id: string; onClick: (id: string) => void }) => (
    <button type="button" onClick={() => onClick(id)}>
      Entfernen {id}
    </button>
  ),
}))

const loadStopClusters = vi.fn<(stops: string[]) => Promise<TreeClusterInListResponse[]>>()
vi.mock('../../api/plans', () => ({
  loadStopClusters: (stops: string[]) => loadStopClusters(stops),
}))

const props = (overrides: Partial<StopPickerProps> = {}): StopPickerProps => ({
  value: ['c1', 'hidden', 'c2'],
  onChange: vi.fn(),
  organizationId: undefined,
  error: undefined,
  form: {
    type: 'update',
    planId: 'p1',
    transporterId: 'v1',
    trailerId: undefined,
    saveDraft: vi.fn(),
  },
  ...overrides,
})

describe('ClusterStopPicker', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    loadStopClusters.mockImplementation((stops) =>
      Promise.resolve(
        stops.filter((id) => id !== 'hidden').map((id) => ({ id }) as TreeClusterInListResponse),
      ),
    )
  })

  it('lists only the clusters the reader can see and keeps the others on removal', async () => {
    const user = userEvent.setup()
    const pickerProps = props()
    render(<ClusterStopPicker {...pickerProps} />)

    await user.click(await screen.findByRole('button', { name: 'Entfernen c1' }))
    expect(screen.queryByText('Entfernen hidden')).not.toBeInTheDocument()

    expect(pickerProps.onChange).toHaveBeenCalledWith(['c2', 'hidden'])
  })

  it('lists no stop before it knows which ones the reader can see', () => {
    loadStopClusters.mockReturnValue(new Promise(() => undefined))
    render(<ClusterStopPicker {...props()} />)

    expect(screen.queryByRole('button', { name: /^Entfernen/ })).not.toBeInTheDocument()
  })

  it('saves the draft and opens the map selection with the plan context', async () => {
    const user = userEvent.setup()
    const pickerProps = props()
    render(<ClusterStopPicker {...pickerProps} />)

    await user.click(screen.getByRole('button', { name: /bewässerungsgruppen hinzufügen/i }))

    expect(pickerProps.form.saveDraft).toHaveBeenCalled()
    expect(navigate).toHaveBeenCalledWith({
      to: '/map/watering-plan/select/cluster',
      search: {
        lat: 54.78,
        lng: 9.43,
        zoom: 14,
        wateringPlanId: 'p1',
        transporterId: 'v1',
        trailerId: undefined,
        clusterIds: ['c1', 'hidden', 'c2'],
        formType: 'update',
      },
    })
  })

  it('names the missing watering group in its own words', () => {
    render(
      <ClusterStopPicker {...props({ value: [], error: 'Wähle mindestens einen Stopp aus.' })} />,
    )

    expect(screen.getByRole('alert')).toHaveTextContent(
      'Wähle mindestens eine Bewässerungsgruppe aus.',
    )
  })
})
