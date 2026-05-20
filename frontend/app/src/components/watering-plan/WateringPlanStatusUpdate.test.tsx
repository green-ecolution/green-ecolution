import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { WateringPlanStatus } from '@green-ecolution/backend-client'
import type { WateringPlan } from '@/api/backendApi'
import { FinishedWateringPlan, CancelWateringPlan } from './WateringPlanStatusUpdate'

vi.mock('../general/cards/SelectedCard', () => ({
  default: ({ id }: { id: string }) => <div data-testid={`selected-card-${id}`}>Cluster {id}</div>,
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
