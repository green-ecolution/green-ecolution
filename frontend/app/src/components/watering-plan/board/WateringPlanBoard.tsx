import { useState } from 'react'
import {
  DndContext,
  DragOverlay,
  KeyboardSensor,
  PointerSensor,
  useDraggable,
  useDroppable,
  useSensor,
  useSensors,
  type DragEndEvent,
  type DragStartEvent,
} from '@dnd-kit/core'
import { useInfiniteQuery, useQuery } from '@tanstack/react-query'
import { CalendarClock, CheckCircle2, Truck } from 'lucide-react'
import {
  Button,
  KanbanBoard,
  KanbanColumn,
  KanbanColumnEmpty,
  KanbanColumnHeader,
  KanbanDropHint,
} from '@green-ecolution/ui'
import { WateringPlanStatus } from '@green-ecolution/backend-client'
import type { User, WateringPlanInList } from '@/api/backendApi'
import {
  userRoleQuery,
  wateringPlanBoardColumnQuery,
  wateringPlanBoardDoneQuery,
} from '@/api/queries'
import {
  dropActionFor,
  dropHintFor,
  type BoardColumnId,
  type DropAction,
} from '@/lib/wateringPlanBoard'
import { useWateringPlanBoardMutations } from '@/hooks/useWateringPlanBoardMutations'
import WateringPlanBoardCard from './WateringPlanBoardCard'
import AssignUsersPopover from './AssignUsersPopover'
import CancelPlanDialog from './CancelPlanDialog'
import CompletePlanDialog from './CompletePlanDialog'
import SuggestionsColumn from './SuggestionsColumn'

interface DragData {
  plan: WateringPlanInList
  column: BoardColumnId
}

const DraggablePlanCard = ({
  plan,
  column,
  users,
}: {
  plan: WateringPlanInList
  column: BoardColumnId
  users: User[]
}) => {
  const { attributes, listeners, setNodeRef, isDragging } = useDraggable({
    id: plan.id,
    data: { plan, column } satisfies DragData,
  })

  return (
    <div ref={setNodeRef} {...listeners} {...attributes} className="touch-none">
      <WateringPlanBoardCard
        plan={plan}
        users={users}
        cardState={isDragging ? 'ghost' : 'idle'}
        assignSlot={
          column === 'planned' ? <AssignUsersPopover plan={plan} users={users} /> : undefined
        }
      />
    </div>
  )
}

const DroppableColumn = ({
  id,
  tone,
  icon,
  title,
  count,
  activeDrag,
  children,
}: {
  id: BoardColumnId
  tone?: 'neutral' | 'active'
  icon: React.ReactNode
  title: string
  count: number
  activeDrag: DragData | null
  children: React.ReactNode
}) => {
  const action = activeDrag ? dropActionFor(activeDrag.column, id) : null
  const disabled = activeDrag !== null && action === null && activeDrag.column !== id
  const { setNodeRef, isOver } = useDroppable({ id, disabled: action === null })

  return (
    <KanbanColumn
      ref={setNodeRef}
      tone={tone}
      state={isOver && action ? 'target' : disabled ? 'dimmed' : 'idle'}
      aria-label={title}
    >
      <KanbanColumnHeader icon={icon} title={title} count={count} />
      {action && <KanbanDropHint label={dropHintFor(action)} />}
      {children}
    </KanbanColumn>
  )
}

const ColumnError = ({ onRetry }: { onRetry: () => void }) => (
  <div className="flex flex-col items-center gap-2 rounded-xl border border-dashed border-red-200 p-4 text-center text-sm text-dark-600">
    Die Einsätze konnten nicht geladen werden.
    <Button type="button" size="sm" variant="outline" className="bg-white" onClick={onRetry}>
      Erneut versuchen
    </Button>
  </div>
)

const WateringPlanBoard = () => {
  const plannedQuery = useQuery(wateringPlanBoardColumnQuery([WateringPlanStatus.Planned]))
  const activeQuery = useQuery(wateringPlanBoardColumnQuery([WateringPlanStatus.Active]))
  const doneQuery = useInfiniteQuery(wateringPlanBoardDoneQuery())
  const { data: usersRes } = useQuery(userRoleQuery('tbz'))
  const { data: plannedRes } = plannedQuery
  const { data: activeRes } = activeQuery

  const { startPlan } = useWateringPlanBoardMutations()
  const [activeDrag, setActiveDrag] = useState<DragData | null>(null)
  const [planToCancel, setPlanToCancel] = useState<WateringPlanInList | null>(null)
  const [planToComplete, setPlanToComplete] = useState<WateringPlanInList | null>(null)

  const sensors = useSensors(
    useSensor(PointerSensor, { activationConstraint: { distance: 8 } }),
    useSensor(KeyboardSensor),
  )

  const planned = plannedRes?.data ?? []
  const active = activeRes?.data ?? []
  const done = doneQuery.data?.pages.flatMap((page) => page.data) ?? []
  const doneTotal = doneQuery.data?.pages[0]?.pagination?.totalRecords ?? done.length
  const users = usersRes?.data ?? []

  const handleDragStart = (event: DragStartEvent) => {
    setActiveDrag(event.active.data.current as DragData)
  }

  const handleDragEnd = (event: DragEndEvent) => {
    const drag = event.active.data.current as DragData | undefined
    setActiveDrag(null)
    if (!drag || !event.over) return
    const action: DropAction | null = dropActionFor(drag.column, event.over.id as BoardColumnId)
    switch (action) {
      case 'start':
        startPlan.mutate(drag.plan)
        break
      case 'cancel':
        setPlanToCancel(drag.plan)
        break
      case 'complete':
        setPlanToComplete(drag.plan)
        break
      case null:
        break
    }
  }

  return (
    <DndContext
      sensors={sensors}
      onDragStart={handleDragStart}
      onDragEnd={handleDragEnd}
      onDragCancel={() => setActiveDrag(null)}
    >
      <KanbanBoard>
        <SuggestionsColumn />
        <DroppableColumn
          id="planned"
          icon={<CalendarClock />}
          title="Geplant"
          count={planned.length}
          activeDrag={activeDrag}
        >
          {plannedQuery.isError && <ColumnError onRetry={() => void plannedQuery.refetch()} />}
          {!plannedQuery.isError && planned.length === 0 && !activeDrag && (
            <KanbanColumnEmpty>
              Keine geplanten Einsätze. Erstellen Sie einen neuen Einsatzplan oder bündeln Sie
              Vorschläge.
            </KanbanColumnEmpty>
          )}
          {planned.map((plan) => (
            <DraggablePlanCard key={plan.id} plan={plan} column="planned" users={users} />
          ))}
        </DroppableColumn>
        <DroppableColumn
          id="active"
          tone="active"
          icon={<Truck />}
          title="Unterwegs"
          count={active.length}
          activeDrag={activeDrag}
        >
          {activeQuery.isError && <ColumnError onRetry={() => void activeQuery.refetch()} />}
          {!activeQuery.isError && active.length === 0 && !activeDrag && (
            <KanbanColumnEmpty>
              Ziehen Sie einen geplanten Einsatz hierher, um ihn zu starten.
            </KanbanColumnEmpty>
          )}
          {active.map((plan) => (
            <DraggablePlanCard key={plan.id} plan={plan} column="active" users={users} />
          ))}
        </DroppableColumn>
        <DroppableColumn
          id="done"
          icon={<CheckCircle2 />}
          title="Erledigt"
          count={doneTotal}
          activeDrag={activeDrag}
        >
          {doneQuery.isError && <ColumnError onRetry={() => void doneQuery.refetch()} />}
          {!doneQuery.isError && done.length === 0 && !activeDrag && (
            <KanbanColumnEmpty>Noch keine erledigten Einsätze.</KanbanColumnEmpty>
          )}
          {done.map((plan) => (
            <WateringPlanBoardCard key={plan.id} plan={plan} users={users} />
          ))}
          {doneQuery.hasNextPage && (
            <Button
              type="button"
              size="sm"
              variant="outline"
              className="bg-white"
              disabled={doneQuery.isFetchingNextPage}
              onClick={() => void doneQuery.fetchNextPage()}
            >
              Mehr laden
            </Button>
          )}
        </DroppableColumn>
      </KanbanBoard>

      <DragOverlay dropAnimation={null}>
        {activeDrag && (
          <WateringPlanBoardCard plan={activeDrag.plan} users={users} cardState="dragging" />
        )}
      </DragOverlay>

      <CancelPlanDialog plan={planToCancel} onClose={() => setPlanToCancel(null)} />
      <CompletePlanDialog plan={planToComplete} onClose={() => setPlanToComplete(null)} />
    </DndContext>
  )
}

export default WateringPlanBoard
