import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen, fireEvent } from '@testing-library/react'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import type { ReactNode } from 'react'
import SensorGeolocationSummary from './SensorGeolocationSummary'

vi.mock('@/api/backendApi', async () => {
  const actual = await vi.importActual<typeof import('@/api/backendApi')>('@/api/backendApi')
  return {
    ...actual,
    treeApi: { listTrees: vi.fn(), getTree: vi.fn() },
  }
})

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
})
