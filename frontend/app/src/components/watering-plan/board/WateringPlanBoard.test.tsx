/* eslint-disable @typescript-eslint/ban-ts-comment */
// @ts-nocheck
import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen } from '@testing-library/react'
import type { ReactNode } from 'react'
import { type Permissions } from '@/lib/auth/permissions'
import { probeManifest } from '@/test/planningManifest'

const permissions = vi.fn((): Permissions => new Set<string>())

vi.mock('@/lib/auth/usePermissions', () => ({
  usePermissions: () => permissions(),
}))

const navigate = vi.fn().mockResolvedValue(undefined)
vi.mock('@tanstack/react-router', () => ({
  useNavigate: () => navigate,
  Link: ({ children }: { children: ReactNode }) => <a>{children}</a>,
}))

const activeManifests = vi.fn(() => [probeManifest('irrigation')])
vi.mock('@/modules/useActiveModules', () => ({
  useActiveModules: () => ({ manifests: activeManifests(), isActive: () => true }),
}))

// Board data hooks — return one planned plan, empty elsewhere.
const plan = {
  id: 'p1',
  date: new Date().toISOString(),
  module: 'irrigation',
  stops: [],
  status: 'planned',
  userIds: [],
  totalDemand: 0,
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
  useWateringPlanBoardMutations: () => ({
    startPlan: { mutate: vi.fn() },
    revertStart: { mutate: vi.fn() },
  }),
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
    activeManifests.mockReturnValue([probeManifest('irrigation')])
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
    expect(screen.queryByText(/Ziehe einen geplanten Einsatzplan/)).not.toBeInTheDocument()
    expect(screen.getByText('Aktuell ist kein Einsatzplan unterwegs.')).toBeInTheDocument()
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

describe('WateringPlanBoard planning module', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    permissions.mockReturnValue(new Set(['watering_plan:read']))
  })

  it('renders the suggestions of the plan module and none of an inactive one', async () => {
    activeManifests.mockReturnValue([probeManifest('irrigation')])
    render(<WateringPlanBoard module="waste" />)

    expect(await screen.findByTestId('suggestions-irrigation')).toBeInTheDocument()
    expect(screen.queryByTestId('suggestions-waste')).not.toBeInTheDocument()
  })

  it('switches between several planning modules above the columns', async () => {
    activeManifests.mockReturnValue([
      probeManifest('irrigation'),
      probeManifest('waste', { labelKey: 'wateringPlan:entity.name' }),
    ])
    render(<WateringPlanBoard module="waste" />)

    expect(await screen.findByTestId('suggestions-waste')).toBeInTheDocument()
    expect(screen.queryByTestId('suggestions-irrigation')).not.toBeInTheDocument()
    expect(screen.getByRole('tab', { selected: true })).toHaveTextContent('Einsatzplan')
    expect(screen.getAllByRole('tab')).toHaveLength(2)
  })

  it('shows no module switcher with a single planning module', () => {
    activeManifests.mockReturnValue([probeManifest('irrigation')])
    render(<WateringPlanBoard />)

    expect(screen.queryByRole('tab')).not.toBeInTheDocument()
  })
})
