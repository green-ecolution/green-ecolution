import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { Suspense, type ReactNode } from 'react'
import { Toaster } from '@green-ecolution/ui'
import { PlanStatus as WateringPlanStatus } from '@green-ecolution/backend-client'
import type { PlanResponse, WateringPlan } from '@/api/backendApi'
import WateringPlanStatusUpdate, {
  FinishedWateringPlan,
  CancelWateringPlan,
} from './WateringPlanStatusUpdate'

vi.mock('../general/cards/SelectedCard', () => ({
  default: ({ id }: { id: string }) => <div data-testid={`selected-card-${id}`}>Cluster {id}</div>,
}))

const getPlan = vi.fn<(...args: unknown[]) => Promise<unknown>>()
const updatePlan = vi.fn<(...args: unknown[]) => Promise<unknown>>()
const listClusters = vi.fn<(...args: unknown[]) => Promise<unknown>>()
vi.mock('@/api/backendApi', async (importOriginal) => {
  const actual = await importOriginal<typeof import('@/api/backendApi')>()
  return {
    ...actual,
    planApi: {
      ...actual.planApi,
      getPlan: (...args: unknown[]) => getPlan(...args),
      updatePlan: (...args: unknown[]) => updatePlan(...args),
    },
    clusterApi: {
      ...actual.clusterApi,
      listClusters: (...args: unknown[]) => listClusters(...args),
    },
  }
})
vi.mock('@tanstack/react-router', () => ({
  useNavigate: () => vi.fn().mockResolvedValue(undefined),
  useRouter: () => ({ invalidate: vi.fn().mockResolvedValue(undefined) }),
  Link: ({ children }: { children: ReactNode }) => <a>{children}</a>,
}))

const PLAN_ID = '0190a8e9-7c4f-7000-8000-000000000001'
const CLUSTER_ID = '0190a8e9-7c4f-7000-8000-000000000010'
const VEHICLE_ID = '0190a8e9-7c4f-7000-8000-000000000020'

const mockLoadedData = {
  id: PLAN_ID,
  date: '2026-03-01T00:00:00Z',
  status: WateringPlanStatus.Active,
  treeclusters: [{ id: CLUSTER_ID, name: 'Cluster A', treeIds: [1, 2, 3, 4, 5] }],
  transporter: { id: VEHICLE_ID },
} as unknown as WateringPlan

const mockPlanResponse = {
  id: PLAN_ID,
  module: 'irrigation',
  date: '2026-03-01T00:00:00Z',
  status: WateringPlanStatus.Active,
  stops: [CLUSTER_ID],
  outcomes: [{ stop: CLUSTER_ID, completed: false, amount: null }],
  transporterCapacityUsed: true,
  totalDemand: 0,
  transporter: { id: VEHICLE_ID },
} as unknown as PlanResponse

describe('FinishedWateringPlan', () => {
  const mockOnSubmit = vi.fn()

  beforeEach(() => {
    vi.clearAllMocks()
  })

  it('save button is enabled with valid default values', async () => {
    render(
      <FinishedWateringPlan
        onSubmit={mockOnSubmit}
        wateringPlanId={PLAN_ID}
        loadedData={mockLoadedData}
      />,
    )

    await waitFor(() => {
      expect(screen.getByRole('button', { name: /speichern/i })).toBeEnabled()
    })
  })

  it('save button stays enabled after editing liters', async () => {
    const user = userEvent.setup()

    render(
      <FinishedWateringPlan
        onSubmit={mockOnSubmit}
        wateringPlanId={PLAN_ID}
        loadedData={mockLoadedData}
      />,
    )

    await waitFor(() => {
      expect(screen.getByRole('button', { name: /speichern/i })).toBeEnabled()
    })

    const input = screen.getByRole('spinbutton')
    await user.clear(input)
    await user.type(input, '500')

    await waitFor(() => {
      expect(screen.getByRole('button', { name: /speichern/i })).toBeEnabled()
    })
  })

  it('save button is disabled when liters is empty', async () => {
    const user = userEvent.setup()

    render(
      <FinishedWateringPlan
        onSubmit={mockOnSubmit}
        wateringPlanId={PLAN_ID}
        loadedData={mockLoadedData}
      />,
    )

    const input = screen.getByRole('spinbutton')
    await user.clear(input)

    await waitFor(() => {
      expect(screen.getByRole('button', { name: /speichern/i })).toBeDisabled()
    })
  })
})

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
    listClusters.mockResolvedValue({ data: mockLoadedData.treeclusters })
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
    expect(payload.planUpdateRequest.stops).toEqual([CLUSTER_ID])
  })
})
