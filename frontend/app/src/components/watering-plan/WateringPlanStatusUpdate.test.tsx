import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { Suspense, type ReactNode } from 'react'
import { Toaster } from '@green-ecolution/ui'
import { PlanStatus as WateringPlanStatus } from '@green-ecolution/backend-client'
import type { PlanResponse } from '@/api/backendApi'
import type { OutcomeInputProps } from '@/modules/types'
import { probeManifest } from '@/test/planningManifest'
import WateringPlanStatusUpdate, { CancelWateringPlan } from './WateringPlanStatusUpdate'

const getPlan = vi.fn<(...args: unknown[]) => Promise<unknown>>()
const updatePlan = vi.fn<(...args: unknown[]) => Promise<unknown>>()
vi.mock('@/api/backendApi', async (importOriginal) => {
  const actual = await importOriginal<typeof import('@/api/backendApi')>()
  return {
    ...actual,
    planApi: {
      ...actual.planApi,
      getPlan: (...args: unknown[]) => getPlan(...args),
      updatePlan: (...args: unknown[]) => updatePlan(...args),
    },
  }
})
const OutcomeInput = ({ onChange }: OutcomeInputProps) => (
  <button
    type="button"
    onClick={() => onChange([{ stop: CLUSTER_ID, completed: true, amount: 300 }], true)}
  >
    Menge melden
  </button>
)
vi.mock('@/modules/useActiveModules', () => ({
  useActiveModules: () => ({
    manifests: [probeManifest('irrigation', { OutcomeInput })],
    isActive: () => true,
  }),
}))
vi.mock('@tanstack/react-router', () => ({
  useNavigate: () => vi.fn().mockResolvedValue(undefined),
  useRouter: () => ({ invalidate: vi.fn().mockResolvedValue(undefined) }),
  Link: ({ children }: { children: ReactNode }) => <a>{children}</a>,
}))

const PLAN_ID = '0190a8e9-7c4f-7000-8000-000000000001'
const CLUSTER_ID = '0190a8e9-7c4f-7000-8000-000000000010'
const VEHICLE_ID = '0190a8e9-7c4f-7000-8000-000000000020'

const HIDDEN_ID = '0190a8e9-7c4f-7000-8000-000000000011'

const mockPlanResponse = {
  id: PLAN_ID,
  module: 'irrigation',
  date: '2026-03-01T00:00:00Z',
  status: WateringPlanStatus.Active,
  stops: [CLUSTER_ID, HIDDEN_ID],
  outcomes: [{ stop: CLUSTER_ID, completed: false, amount: null }],
  transporterCapacityUsed: true,
  totalDemand: 0,
  transporter: { id: VEHICLE_ID },
} as unknown as PlanResponse

describe('CancelWateringPlan', () => {
  const mockOnSubmit = vi.fn()

  beforeEach(() => {
    vi.clearAllMocks()
  })

  it('save button is disabled initially with empty note', async () => {
    render(<CancelWateringPlan onSubmit={mockOnSubmit} />)

    await waitFor(() => {
      expect(screen.getByRole('button', { name: /speichern/i })).toBeDisabled()
    })
  })

  it('save button is enabled after typing a cancellation note', async () => {
    const user = userEvent.setup()

    render(<CancelWateringPlan onSubmit={mockOnSubmit} />)

    const textarea = screen.getByPlaceholderText(/warum wurde der einsatzplan abgebrochen/i)
    await user.type(textarea, 'Schlechtes Wetter')

    await waitFor(() => {
      expect(screen.getByRole('button', { name: /speichern/i })).toBeEnabled()
    })
  })
})

function createWrapper() {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false }, mutations: { retry: false, throwOnError: false } },
  })
  return ({ children }: { children: ReactNode }) => (
    <QueryClientProvider client={queryClient}>
      <Suspense fallback={<p>lädt</p>}>{children}</Suspense>
      <Toaster />
    </QueryClientProvider>
  )
}

describe('WateringPlanStatusUpdate — Nicht angetreten', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    getPlan.mockResolvedValue(mockPlanResponse)
  })

  const selectNotCompleted = async (user: ReturnType<typeof userEvent.setup>) => {
    render(<WateringPlanStatusUpdate wateringPlanId={PLAN_ID} />, { wrapper: createWrapper() })

    const trigger = await screen.findByRole('combobox', { name: /status des einsatzplans/i })
    await user.click(trigger)
    await user.click(await screen.findByRole('option', { name: /nicht angetreten/i }))
  }

  it('asks for a reason instead of saving straight away', async () => {
    const user = userEvent.setup()
    await selectNotCompleted(user)

    const textarea = await screen.findByPlaceholderText(
      /warum wurde der einsatzplan nicht angetreten/i,
    )
    expect(textarea).toBeInTheDocument()
    expect(screen.getByRole('button', { name: /speichern/i })).toBeDisabled()
  })

  it('submits the reason as cancellationNote', async () => {
    const user = userEvent.setup()
    updatePlan.mockResolvedValueOnce({ ...mockPlanResponse })
    await selectNotCompleted(user)

    const textarea = await screen.findByPlaceholderText(
      /warum wurde der einsatzplan nicht angetreten/i,
    )
    await user.type(textarea, 'Fahrzeug defekt')
    await user.click(screen.getByRole('button', { name: /speichern/i }))

    await waitFor(() => expect(updatePlan).toHaveBeenCalledTimes(1))
    const [payload] = updatePlan.mock.calls[0] as [
      {
        planId: string
        planUpdateRequest: { status: string; cancellationNote: string; stops: string[] }
      },
    ]
    expect(payload.planId).toBe(PLAN_ID)
    expect(payload.planUpdateRequest.status).toBe(WateringPlanStatus.NotCompleted)
    expect(payload.planUpdateRequest.cancellationNote).toBe('Fahrzeug defekt')
    expect(payload.planUpdateRequest.stops).toEqual([CLUSTER_ID, HIDDEN_ID])
  })
})

describe('WateringPlanStatusUpdate — Beendet', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    getPlan.mockResolvedValue(mockPlanResponse)
  })

  it('finishes with the outcomes of the plan module and completes the hidden stops', async () => {
    const user = userEvent.setup()
    updatePlan.mockResolvedValueOnce({ ...mockPlanResponse })
    render(<WateringPlanStatusUpdate wateringPlanId={PLAN_ID} />, { wrapper: createWrapper() })

    await user.click(await screen.findByRole('combobox', { name: /status des einsatzplans/i }))
    await user.click(await screen.findByRole('option', { name: /beendet/i }))
    expect(screen.getByRole('button', { name: /speichern/i })).toBeDisabled()

    await user.click(await screen.findByRole('button', { name: 'Menge melden' }))
    await user.click(screen.getByRole('button', { name: /speichern/i }))

    await waitFor(() => expect(updatePlan).toHaveBeenCalledTimes(1))
    const [payload] = updatePlan.mock.calls[0] as [
      { planUpdateRequest: { status: string; outcomes: unknown[] } },
    ]
    expect(payload.planUpdateRequest.status).toBe(WateringPlanStatus.Finished)
    expect(payload.planUpdateRequest.outcomes).toEqual([
      { stop: CLUSTER_ID, completed: true, amount: 300 },
      { stop: HIDDEN_ID, completed: true, amount: null },
    ])
  })
})
