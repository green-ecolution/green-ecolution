import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import type { ReactNode } from 'react'
import { Toaster } from '@green-ecolution/ui'
import { WateringPlanStatus } from '@green-ecolution/backend-client'
import CompletePlanDialog from './CompletePlanDialog'
import type { WateringPlanInList } from '@/api/backendApi'

const updateWateringPlan = vi.fn()
vi.mock('@/api/backendApi', async (importOriginal) => {
  const actual = await importOriginal<typeof import('@/api/backendApi')>()
  return {
    ...actual,
    wateringPlanApi: { ...actual.wateringPlanApi, updateWateringPlan: (...args: unknown[]) => updateWateringPlan(...args) },
  }
})
vi.mock('@tanstack/react-router', () => ({
  useNavigate: () => vi.fn().mockResolvedValue(undefined),
  Link: ({ children }: { children: ReactNode }) => <a>{children}</a>,
}))

const plan: WateringPlanInList = {
  id: '0190a8e9-7c4f-7000-8000-000000000000',
  createdAt: '2026-07-01T00:00:00Z',
  updatedAt: '2026-07-01T00:00:00Z',
  date: '2026-07-08T00:00:00Z',
  description: 'Innenstadt',
  status: WateringPlanStatus.Active,
  distance: 1000,
  totalWaterRequired: 4200,
  cancellationNote: '',
  transporter: { id: 'v-1', numberPlate: 'FL-GE-A01' } as WateringPlanInList['transporter'],
  treeclusters: [],
  userIds: [],
}

function createWrapper() {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false }, mutations: { retry: false, throwOnError: false } },
  })
  return ({ children }: { children: ReactNode }) => (
    <QueryClientProvider client={queryClient}>
      {children}
      <Toaster />
    </QueryClientProvider>
  )
}

describe('CompletePlanDialog', () => {
  beforeEach(() => vi.clearAllMocks())

  it('submits a cancellation with note', async () => {
    const user = userEvent.setup()
    updateWateringPlan.mockResolvedValueOnce({})
    render(<CompletePlanDialog plan={plan} onClose={vi.fn()} />, { wrapper: createWrapper() })

    await user.click(screen.getByRole('radio', { name: /abgebrochen/i }))
    await user.type(screen.getByLabelText(/grund des abbruchs/i), 'Fahrzeug defekt')
    await user.click(screen.getByRole('button', { name: /einsatz abbrechen/i }))

    await waitFor(() => expect(updateWateringPlan).toHaveBeenCalledTimes(1))
    const request = updateWateringPlan.mock.calls[0][0] as {
      wateringPlanUpdateRequest: { status: string; cancellationNote: string }
    }
    expect(request.wateringPlanUpdateRequest.status).toBe(WateringPlanStatus.Canceled)
    expect(request.wateringPlanUpdateRequest.cancellationNote).toBe('Fahrzeug defekt')
  })

  it('renders the evaluation form in finished mode', () => {
    render(<CompletePlanDialog plan={plan} onClose={vi.fn()} />, { wrapper: createWrapper() })
    expect(screen.getByText(/wasservergabe pro bewässerungsgruppe/i)).toBeInTheDocument()
  })
})
