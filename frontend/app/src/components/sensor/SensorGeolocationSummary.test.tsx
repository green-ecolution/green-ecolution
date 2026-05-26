import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen, fireEvent, act } from '@testing-library/react'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import type { ReactNode } from 'react'
import type { TreeResponse } from '@green-ecolution/backend-client'
import SensorGeolocationSummary from './SensorGeolocationSummary'
import { treeApi } from '@/api/backendApi'

vi.mock('@/api/backendApi', async () => {
  const actual = await vi.importActual<typeof import('@/api/backendApi')>('@/api/backendApi')
  return {
    ...actual,
    treeApi: {
      listTrees: vi.fn(),
      getTree: vi.fn(),
      getNearestTrees: vi.fn(),
    },
  }
})

function makeTree(overrides: Partial<TreeResponse> = {}): TreeResponse {
  return {
    id: 'tree-1',
    species: 'Eiche',
    number: 'T-001',
    latitude: 0,
    longitude: 0,
    wateringStatus: 'unknown',
    createdAt: '2024-01-01T00:00:00Z',
    updatedAt: '2024-01-01T00:00:00Z',
    description: '',
    plantingYear: 2000,
    sensor: null,
    ...overrides,
  } as TreeResponse
}

function renderWithClient(ui: ReactNode) {
  const qc = new QueryClient({ defaultOptions: { queries: { retry: false } } })
  return render(<QueryClientProvider client={qc}>{ui}</QueryClientProvider>)
}

const baseProps = {
  sensorId: 'sensor-1',
  position: null,
  status: 'idle' as const,
  errorMessage: null,
  onScanAgain: vi.fn(),
  onRelocate: vi.fn(),
}

describe('SensorGeolocationSummary', () => {
  beforeEach(() => vi.clearAllMocks())

  it('renders "Anderen Baum auswählen" button', () => {
    renderWithClient(<SensorGeolocationSummary {...baseProps} />)
    expect(screen.getByRole('button', { name: /Anderen Baum auswählen/i })).toBeInTheDocument()
  })

  it('opens the picker sheet on click', () => {
    renderWithClient(<SensorGeolocationSummary {...baseProps} />)
    fireEvent.click(screen.getByRole('button', { name: /Anderen Baum auswählen/i }))
    expect(screen.getByRole('searchbox')).toBeInTheDocument()
  })

  it('renders "Ausgewählter Baum" card after picker selection outside nearest list', async () => {
    vi.useFakeTimers({ shouldAdvanceTime: true })
    // eslint-disable-next-line @typescript-eslint/unbound-method
    vi.mocked(treeApi.listTrees).mockResolvedValue({
      data: [makeTree({ id: 'tree-x', species: 'Buche', number: 'T-XX' })],
      pagination: { totalRecords: 1, totalPages: 1, currentPage: 1, perPage: 20 },
    })
    // eslint-disable-next-line @typescript-eslint/unbound-method
    vi.mocked(treeApi.getTree).mockResolvedValue(
      makeTree({ id: 'tree-x', species: 'Buche', number: 'T-XX' }),
    )

    renderWithClient(<SensorGeolocationSummary {...baseProps} />)
    fireEvent.click(screen.getByRole('button', { name: /Anderen Baum auswählen/i }))
    fireEvent.change(screen.getByRole('searchbox'), { target: { value: 'Buche' } })
    await act(async () => {
      await vi.advanceTimersByTimeAsync(350)
    })
    const row = await screen.findByRole('button', { name: /Buche/ })
    fireEvent.click(row)
    await act(async () => {
      await vi.advanceTimersByTimeAsync(0)
    })

    expect(await screen.findByText(/Ausgewählter Baum/i)).toBeInTheDocument()
    expect(screen.getByText('T-XX')).toBeInTheDocument()
    vi.useRealTimers()
  })
})
