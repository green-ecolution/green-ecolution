import { describe, it, expect, vi } from 'vitest'
import { render, screen, fireEvent } from '@testing-library/react'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import type { ReactNode } from 'react'
import SensorTreeSearchResults from './SensorTreeSearchResults'
import { treeApi } from '@/api/backendApi'
import type { TreeResponse, ListResponseTreeResponse } from '@green-ecolution/backend-client'

vi.mock('@/api/backendApi', async () => {
  const actual = await vi.importActual<typeof import('@/api/backendApi')>('@/api/backendApi')
  return { ...actual, treeApi: { listTrees: vi.fn() } }
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
  }
}

function makeListResponse(trees: TreeResponse[]): ListResponseTreeResponse {
  return {
    data: trees,
    pagination: {
      totalRecords: trees.length,
      totalPages: 1,
      currentPage: 1,
      perPage: 20,
    },
  }
}

function renderWithClient(ui: ReactNode) {
  const qc = new QueryClient({ defaultOptions: { queries: { retry: false } } })
  return render(<QueryClientProvider client={qc}>{ui}</QueryClientProvider>)
}

describe('SensorTreeSearchResults', () => {
  it('shows idle hint when q is empty', () => {
    renderWithClient(<SensorTreeSearchResults q="" selectedTreeId={null} onSelect={vi.fn()} />)
    expect(screen.getByText(/Tippe Baumnummer oder Baumart/i)).toBeInTheDocument()
  })

  it('shows idle hint when q is only whitespace', () => {
    renderWithClient(<SensorTreeSearchResults q="   " selectedTreeId={null} onSelect={vi.fn()} />)
    expect(screen.getByText(/Tippe Baumnummer oder Baumart/i)).toBeInTheDocument()
  })

  it('shows empty-state when q has no matches', async () => {
    // eslint-disable-next-line @typescript-eslint/unbound-method
    vi.mocked(treeApi.listTrees).mockResolvedValueOnce(makeListResponse([]))

    renderWithClient(<SensorTreeSearchResults q="xyz" selectedTreeId={null} onSelect={vi.fn()} />)

    expect(await screen.findByText(/Keine Bäume gefunden/i)).toBeInTheDocument()
  })

  it('renders rows and calls onSelect for unassigned trees', async () => {
    // eslint-disable-next-line @typescript-eslint/unbound-method
    vi.mocked(treeApi.listTrees).mockResolvedValueOnce(
      makeListResponse([makeTree({ id: 'tree-a', species: 'Eiche', number: 'T-1', sensor: null })]),
    )

    const onSelect = vi.fn()
    renderWithClient(<SensorTreeSearchResults q="T-" selectedTreeId={null} onSelect={onSelect} />)

    const row = await screen.findByRole('button', { name: /Eiche/ })
    fireEvent.click(row)
    expect(onSelect).toHaveBeenCalledWith('tree-a')
  })

  it('marks rows whose tree has a sensor as not selectable', async () => {
    // eslint-disable-next-line @typescript-eslint/unbound-method
    vi.mocked(treeApi.listTrees).mockResolvedValueOnce(
      makeListResponse([
        makeTree({ id: 'tree-a', species: 'Eiche', number: 'T-1', sensor: null }),
        makeTree({
          id: 'tree-b',
          species: 'Buche',
          number: 'T-2',
          sensor: {
            id: 'eui',
            status: 'online',
            createdAt: '',
            updatedAt: '',
            model: { id: 'model-1', name: 'EcoDrizzler' },
            sensorType: 'lorawan',
          },
        }),
      ]),
    )

    const onSelect = vi.fn()
    renderWithClient(<SensorTreeSearchResults q="T-" selectedTreeId={null} onSelect={onSelect} />)

    const buche = await screen.findByRole('button', { name: /Buche/ })
    expect(buche).toHaveAttribute('aria-disabled', 'true')
    expect(buche).toBeDisabled()
    fireEvent.click(buche)
    expect(onSelect).not.toHaveBeenCalled()
    expect(screen.getByText(/Sensor zugeordnet/i)).toBeInTheDocument()
  })
})
