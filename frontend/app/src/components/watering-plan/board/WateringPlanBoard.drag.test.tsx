/* eslint-disable @typescript-eslint/ban-ts-comment */
// @ts-nocheck
import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen, act } from '@testing-library/react'
import type { ReactNode } from 'react'

vi.mock('@/lib/auth/usePermissions', () => ({
  usePermissions: () => new Set(['watering_plan:update']),
}))

vi.mock('@tanstack/react-router', () => ({
  useNavigate: () => vi.fn().mockResolvedValue(undefined),
  Link: ({ children }: { children: ReactNode }) => <a>{children}</a>,
}))

vi.mock('@/modules/useActiveModules', async () => {
  const { probeManifest } = await import('@/test/planningManifest')
  return {
    useActiveModules: () => ({ manifests: [probeManifest('irrigation')], isActive: () => true }),
  }
})

const activePlan = {
  id: 'p1',
  date: new Date().toISOString(),
  module: 'irrigation',
  stops: [],
  status: 'active',
  userIds: [],
  totalDemand: 0,
  transporter: { numberPlate: 'TEST-1' },
}

interface DragData {
  plan: typeof activePlan
  column: 'planned' | 'active' | 'done'
}
interface DragStartEvent {
  active: { data: { current: DragData } }
}
type DragEndEvent = DragStartEvent & { over: { id: string } | null }

// dnd-kit needs the DOM rects a real drag measures, which jsdom does not provide;
// the mock hands the test the DndContext callbacks so a drop can be replayed.
const notRendered = () => {
  throw new Error('DndContext has not rendered yet')
}
const dnd: {
  onDragStart: (event: DragStartEvent) => void
  onDragEnd: (event: DragEndEvent) => void
} = { onDragStart: notRendered, onDragEnd: notRendered }
const droppableCalls: { id: string; disabled: boolean }[] = []

vi.mock('@dnd-kit/core', () => ({
  DndContext: ({
    children,
    onDragStart,
    onDragEnd,
  }: {
    children: ReactNode
    onDragStart: (event: DragStartEvent) => void
    onDragEnd: (event: DragEndEvent) => void
  }) => {
    dnd.onDragStart = onDragStart
    dnd.onDragEnd = onDragEnd
    return <>{children}</>
  },
  DragOverlay: ({ children }: { children: ReactNode }) => <>{children}</>,
  KeyboardSensor: {},
  PointerSensor: {},
  useSensor: () => ({}),
  useSensors: () => [],
  useDraggable: () => ({
    attributes: {},
    listeners: {},
    setNodeRef: vi.fn(),
    isDragging: false,
  }),
  useDroppable: ({ id, disabled }: { id: string; disabled: boolean }) => {
    droppableCalls.push({ id, disabled })
    return { setNodeRef: vi.fn(), isOver: false }
  },
}))

vi.mock('@tanstack/react-query', async (importOriginal) => {
  const actual = await importOriginal<typeof import('@tanstack/react-query')>()
  return {
    ...actual,
    useQuery: (opts: { queryKey: unknown[] }) => {
      const key = JSON.stringify(opts.queryKey)
      if (key.includes('active')) return { data: { data: [activePlan] }, isError: false }
      return { data: { data: [] }, isError: false }
    },
    useInfiniteQuery: () => ({
      data: { pages: [{ data: [], pagination: { totalRecords: 0 } }] },
      isError: false,
      hasNextPage: false,
      fetchNextPage: vi.fn(),
    }),
  }
})

const revertStart = { mutate: vi.fn() }
const startPlan = { mutate: vi.fn() }
vi.mock('@/hooks/useWateringPlanBoardMutations', () => ({
  useWateringPlanBoardMutations: () => ({ revertStart, startPlan }),
}))

vi.mock('./AssignUsersPopover', () => ({ default: () => <div /> }))

const { default: WateringPlanBoard } = await import('./WateringPlanBoard')

const drag: DragStartEvent = {
  active: { data: { current: { plan: activePlan, column: 'active' } } },
}
const lastDroppableCall = (id: string) => droppableCalls.filter((call) => call.id === id).pop()

describe('dragging a plan from Unterwegs back to Geplant', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    droppableCalls.length = 0
  })

  it('offers Geplant as an active drop target', () => {
    render(<WateringPlanBoard />)
    act(() => dnd.onDragStart(drag))

    expect(lastDroppableCall('planned')?.disabled).toBe(false)
    expect(screen.getByLabelText('Geplant').className).not.toContain('opacity-40')
  })

  it('shows the undo-start drop hint on the Geplant column', () => {
    render(<WateringPlanBoard />)
    act(() => dnd.onDragStart(drag))

    expect(screen.getByText('Start zurücknehmen')).toBeInTheDocument()
  })

  it('reverts the start when the card is dropped on Geplant', () => {
    render(<WateringPlanBoard />)
    act(() => dnd.onDragStart(drag))
    act(() => dnd.onDragEnd({ ...drag, over: { id: 'planned' } }))

    expect(revertStart.mutate).toHaveBeenCalledWith(activePlan)
    expect(startPlan.mutate).not.toHaveBeenCalled()
  })
})
