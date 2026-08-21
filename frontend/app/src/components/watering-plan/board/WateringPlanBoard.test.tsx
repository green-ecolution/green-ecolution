/* eslint-disable @typescript-eslint/ban-ts-comment */
// @ts-nocheck
import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen } from '@testing-library/react'
import type { ReactNode } from 'react'
import { type Permissions } from '@/lib/auth/permissions'

const permissions = vi.fn((): Permissions => new Set<string>())

vi.mock('@/lib/auth/usePermissions', () => ({
  usePermissions: () => permissions(),
}))

vi.mock('@tanstack/react-router', () => ({
  useNavigate: () => vi.fn().mockResolvedValue(undefined),
  Link: ({ children }: { children: ReactNode }) => <a>{children}</a>,
}))

// Board data hooks — return one planned plan, empty elsewhere.
const plan = {
  id: 'p1',
  date: new Date().toISOString(),
  treeclusters: [],
  status: 'planned',
  userIds: [],
  totalWaterRequired: 0,
  transporter: { numberPlate: 'TEST-1' },
}
vi.mock('@tanstack/react-query', async (importOriginal) => {
  const actual = await importOriginal<typeof import('@tanstack/react-query')>()
  return {
    ...actual,
    useQuery: (opts: { queryKey: unknown[] }) => {
      const key = JSON.stringify(opts.queryKey)
      if (key.includes('planned'))
        return { data: { data: [plan] }, isError: false, refetch: vi.fn() }
      return { data: { data: [] }, isError: false, refetch: vi.fn() }
    },
    useInfiniteQuery: () => ({
      data: { pages: [{ data: [], pagination: { totalRecords: 0 } }] },
      isError: false,
      hasNextPage: false,
      fetchNextPage: vi.fn(),
    }),
  }
})

vi.mock('@/hooks/useWateringPlanBoardMutations', () => ({
  useWateringPlanBoardMutations: () => ({ startPlan: { mutate: vi.fn() } }),
}))

// AssignUsersPopover is watering_plan:update only; render a marker we can assert on.
vi.mock('./AssignUsersPopover', () => ({
  default: () => <div data-testid="assign-users" />,
}))

const { default: WateringPlanBoard } = await import('./WateringPlanBoard')

describe('WateringPlanBoard modify gating', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    permissions.mockReturnValue(new Set<string>())
  })

  it('hides the assign-users control without watering_plan:update', () => {
    permissions.mockReturnValue(new Set(['watering_plan:read']))
    render(<WateringPlanBoard />)
    expect(screen.queryByTestId('assign-users')).not.toBeInTheDocument()
  })

  it('shows the assign-users control with watering_plan:update', () => {
    permissions.mockReturnValue(new Set(['watering_plan:update']))
    render(<WateringPlanBoard />)
    expect(screen.getByTestId('assign-users')).toBeInTheDocument()
  })

  it('omits the drag instruction without watering_plan:update', () => {
    permissions.mockReturnValue(new Set(['watering_plan:read']))
    render(<WateringPlanBoard />)
    expect(screen.queryByText(/Ziehen Sie/)).not.toBeInTheDocument()
    expect(screen.getByText('Aktuell ist kein Einsatz unterwegs.')).toBeInTheDocument()
  })

  it('renders planned plans as plain cards without watering_plan:update', () => {
    permissions.mockReturnValue(new Set(['watering_plan:read']))
    const { container } = render(<WateringPlanBoard />)
    expect(container.querySelector('[aria-roledescription="draggable"]')).toBeNull()
  })

  it('renders planned plans as draggable cards with watering_plan:update', () => {
    permissions.mockReturnValue(new Set(['watering_plan:update']))
    const { container } = render(<WateringPlanBoard />)
    expect(container.querySelector('[aria-roledescription="draggable"]')).not.toBeNull()
  })
})
