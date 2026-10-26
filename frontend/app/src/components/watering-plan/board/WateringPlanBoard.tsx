import { Suspense, useState } from 'react'
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
import { useNavigate } from '@tanstack/react-router'
import { CalendarClock, CheckCircle2, Truck } from 'lucide-react'
import {
  Button,
  KanbanBoard,
  KanbanColumn,
  KanbanColumnEmpty,
  KanbanColumnHeader,
  KanbanDropHint,
  Tabs,
  TabsList,
  TabsTrigger,
} from '@green-ecolution/ui'
import { useTranslation } from 'react-i18next'
import { PlanStatus as WateringPlanStatus } from '@green-ecolution/backend-client'
import type { PlanInListResponse } from '@green-ecolution/backend-client'
import type { User } from '@/api/backendApi'
import { userQueries, wateringPlanQueries } from '@/api/queries'
import {
  dropActionFor,
  dropHintFor,
  type BoardColumnId,
  type DropAction,
} from '@/lib/wateringPlanBoard'
import { useWateringPlanBoardMutations } from '@/hooks/useWateringPlanBoardMutations'
import { usePlanPermission } from '@/modules/usePlanPermission'
import { useActiveModules } from '@/modules/useActiveModules'
import { planModule, planningManifests, type PlanningManifest } from '@/modules/planning'
import type { ModulePlanning } from '@/modules/types'
import WateringPlanBoardCard from './WateringPlanBoardCard'
import AssignUsersPopover from './AssignUsersPopover'
import CancelPlanDialog from './CancelPlanDialog'
import CompletePlanDialog from './CompletePlanDialog'

interface DragData {
  plan: PlanInListResponse
  column: BoardColumnId
}

interface PlanCardProps {
  plan: PlanInListResponse
  planning: ModulePlanning
  column: BoardColumnId
  users: User[]
}

const DraggablePlanCard = ({ plan, planning, column, users }: PlanCardProps) => {
  const { attributes, listeners, setNodeRef, isDragging } = useDraggable({
    id: plan.id,
    data: { plan, column } satisfies DragData,
  })

  return (
    <div ref={setNodeRef} {...listeners} {...attributes} className="touch-none">
      <WateringPlanBoardCard
        plan={plan}
        planning={planning}
        users={users}
        cardState={isDragging ? 'ghost' : 'idle'}
        assignSlot={
          column === 'planned' ? <AssignUsersPopover plan={plan} users={users} /> : undefined
        }
      />
    </div>
  )
}

const PlanCard = ({ canModify, ...props }: PlanCardProps & { canModify: boolean }) =>
  canModify ? (
    <DraggablePlanCard {...props} />
  ) : (
    <WateringPlanBoardCard plan={props.plan} planning={props.planning} users={props.users} />
  )

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
  const { t } = useTranslation('wateringPlan')

  return (
    <KanbanColumn
      ref={setNodeRef}
      tone={tone}
      state={isOver && action ? 'target' : disabled ? 'dimmed' : 'idle'}
      aria-label={title}
    >
      <KanbanColumnHeader icon={icon} title={title} count={count} />
      {action && <KanbanDropHint label={dropHintFor(action, t)} />}
      {children}
    </KanbanColumn>
  )
}

const ColumnError = ({ onRetry }: { onRetry: () => void }) => {
  const { t } = useTranslation(['wateringPlan', 'common'])
  return (
    <div className="flex flex-col items-center gap-2 rounded-xl border border-dashed border-red-200 p-4 text-center text-sm text-dark-600">
      {t('board.columnError.message')}
      <Button type="button" size="sm" variant="outline" className="bg-white" onClick={onRetry}>
        {t('common:actions.retry')}
      </Button>
    </div>
  )
}

const ModuleSwitcher = ({
  current,
  manifests,
}: {
  current: string
  manifests: PlanningManifest[]
}) => {
  const navigate = useNavigate()
  const { t, i18n } = useTranslation('wateringPlan')

  return (
    <Tabs
      value={current}
      onValueChange={(module) => {
        navigate({ to: '/watering-plans', search: { module } }).catch((error) =>
          console.error('Navigation failed:', error),
        )
      }}
      className="mb-4"
    >
      <TabsList aria-label={t('board.moduleSwitcherAriaLabel')}>
        {manifests.map(({ key, planning }) => (
          <TabsTrigger key={key} value={key}>
            {i18n.t(planning.labelKey)}
          </TabsTrigger>
        ))}
      </TabsList>
    </Tabs>
  )
}

const WateringPlanBoard = ({ module }: { module?: string }) => {
  const { manifests } = useActiveModules()
  const planManifest = planModule({ module }, manifests)
  if (!planManifest) return null
  const planningModules = planningManifests(manifests)

  return (
    <>
      {planningModules.length > 1 && (
        <ModuleSwitcher current={planManifest.key} manifests={planningModules} />
      )}
      <ModuleBoard
        key={planManifest.key}
        module={planManifest.key}
        planning={planManifest.planning}
      />
    </>
  )
}

const ModuleBoard = ({ module, planning }: { module: string; planning: ModulePlanning }) => {
  const plannedQuery = useQuery(
    wateringPlanQueries.boardColumn(module, [WateringPlanStatus.Planned]),
  )
  const activeQuery = useQuery(wateringPlanQueries.boardColumn(module, [WateringPlanStatus.Active]))
  const doneQuery = useInfiniteQuery(wateringPlanQueries.boardDone(module))
  const { data: usersRes } = useQuery(userQueries.list({ page: 1, perPage: 100 }))
  const { data: plannedRes } = plannedQuery
  const { data: activeRes } = activeQuery

  const { startPlan, revertStart } = useWateringPlanBoardMutations(planning)
  const { SuggestionsColumn } = planning
  const canModify = usePlanPermission(planning, 'update')
  const canCreate = usePlanPermission(planning, 'create')
  const { t } = useTranslation('wateringPlan')
  const [activeDrag, setActiveDrag] = useState<DragData | null>(null)
  const [planToCancel, setPlanToCancel] = useState<PlanInListResponse | null>(null)
  const [planToComplete, setPlanToComplete] = useState<PlanInListResponse | null>(null)

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
      case 'revertStart':
        revertStart.mutate(drag.plan)
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
      <KanbanBoard className="justify-center-safe pt-0.5">
        <Suspense fallback={null}>
          <SuggestionsColumn />
        </Suspense>
        <DroppableColumn
          id="planned"
          icon={<CalendarClock />}
          title={t('board.column.planned')}
          count={planned.length}
          activeDrag={activeDrag}
        >
          {plannedQuery.isError && <ColumnError onRetry={() => void plannedQuery.refetch()} />}
          {!plannedQuery.isError && planned.length === 0 && !activeDrag && (
            <KanbanColumnEmpty>
              {canCreate ? t('board.column.plannedEmptyCanCreate') : t('board.column.plannedEmpty')}
            </KanbanColumnEmpty>
          )}
          {planned.map((plan) => (
            <PlanCard
              key={plan.id}
              plan={plan}
              planning={planning}
              column="planned"
              users={users}
              canModify={canModify}
            />
          ))}
        </DroppableColumn>
        <DroppableColumn
          id="active"
          tone="active"
          icon={<Truck />}
          title={t('board.column.active')}
          count={active.length}
          activeDrag={activeDrag}
        >
          {activeQuery.isError && <ColumnError onRetry={() => void activeQuery.refetch()} />}
          {!activeQuery.isError && active.length === 0 && !activeDrag && (
            <KanbanColumnEmpty>
              {canModify ? t('board.column.activeEmptyCanModify') : t('board.column.activeEmpty')}
            </KanbanColumnEmpty>
          )}
          {active.map((plan) => (
            <PlanCard
              key={plan.id}
              plan={plan}
              planning={planning}
              column="active"
              users={users}
              canModify={canModify}
            />
          ))}
        </DroppableColumn>
        <DroppableColumn
          id="done"
          icon={<CheckCircle2 />}
          title={t('board.column.done')}
          count={doneTotal}
          activeDrag={activeDrag}
        >
          {doneQuery.isError && <ColumnError onRetry={() => void doneQuery.refetch()} />}
          {!doneQuery.isError && done.length === 0 && !activeDrag && (
            <KanbanColumnEmpty>{t('board.column.doneEmpty')}</KanbanColumnEmpty>
          )}
          {done.map((plan) => (
            <WateringPlanBoardCard key={plan.id} plan={plan} planning={planning} users={users} />
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
              {t('board.column.loadMore')}
            </Button>
          )}
        </DroppableColumn>
      </KanbanBoard>

      <DragOverlay dropAnimation={null}>
        {activeDrag && (
          <WateringPlanBoardCard
            plan={activeDrag.plan}
            planning={planning}
            users={users}
            cardState="dragging"
          />
        )}
      </DragOverlay>

      <CancelPlanDialog
        plan={planToCancel}
        planning={planning}
        onClose={() => setPlanToCancel(null)}
      />
      <CompletePlanDialog
        plan={planToComplete}
        planning={planning}
        onClose={() => setPlanToComplete(null)}
      />
    </DndContext>
  )
}

export default WateringPlanBoard
