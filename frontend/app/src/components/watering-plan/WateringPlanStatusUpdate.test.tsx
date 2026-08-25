import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { Suspense, type ReactNode } from 'react'
import { Toaster } from '@green-ecolution/ui'
import { WateringPlanStatus } from '@green-ecolution/backend-client'
import type { WateringPlan } from '@/api/backendApi'
import WateringPlanStatusUpdate, {
  FinishedWateringPlan,
  CancelWateringPlan,
} from './WateringPlanStatusUpdate'

vi.mock('../general/cards/SelectedCard', () => ({
  default: ({ id }: { id: string }) => <div data-testid={`selected-card-${id}`}>Cluster {id}</div>,
}))

const getWateringPlan = vi.fn<(...args: unknown[]) => Promise<unknown>>()
const updateWateringPlan = vi.fn<(...args: unknown[]) => Promise<unknown>>()
vi.mock('@/api/backendApi', async (importOriginal) => {
  const actual = await importOriginal<typeof import('@/api/backendApi')>()
  return {
    ...actual,
    wateringPlanApi: {
      ...actual.wateringPlanApi,
      getWateringPlan: (...args: unknown[]) => getWateringPlan(...args),
      updateWateringPlan: (...args: unknown[]) => updateWateringPlan(...args),
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

    const textarea = screen.getByPlaceholderText(/warum wurde der einsatz abgebrochen/i)
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
    getWateringPlan.mockResolvedValue(mockLoadedData)
  })

  const selectNotCompleted = async (user: ReturnType<typeof userEvent.setup>) => {
    render(<WateringPlanStatusUpdate wateringPlanId={PLAN_ID} />, { wrapper: createWrapper() })

    const trigger = await screen.findByRole('combobox', { name: /status des einsatzes/i })
    await user.click(trigger)
    await user.click(await screen.findByRole('option', { name: /nicht angetreten/i }))
  }

  it('asks for a reason instead of saving straight away', async () => {
    const user = userEvent.setup()
    await selectNotCompleted(user)

    const textarea = await screen.findByPlaceholderText(/warum wurde der einsatz nicht angetreten/i)
    expect(textarea).toBeInTheDocument()
    expect(screen.getByRole('button', { name: /speichern/i })).toBeDisabled()
  })

  it('submits the reason as cancellationNote', async () => {
    const user = userEvent.setup()
    updateWateringPlan.mockResolvedValueOnce({ ...mockLoadedData })
    await selectNotCompleted(user)

    const textarea = await screen.findByPlaceholderText(/warum wurde der einsatz nicht angetreten/i)
    await user.type(textarea, 'Fahrzeug defekt')
    await user.click(screen.getByRole('button', { name: /speichern/i }))

    await waitFor(() => expect(updateWateringPlan).toHaveBeenCalledTimes(1))
    const [payload] = updateWateringPlan.mock.calls[0] as [
      { wateringPlanUpdateRequest: { status: string; cancellationNote: string } },
    ]
    expect(payload.wateringPlanUpdateRequest.status).toBe(WateringPlanStatus.NotCompeted)
    expect(payload.wateringPlanUpdateRequest.cancellationNote).toBe('Fahrzeug defekt')
  })
})
