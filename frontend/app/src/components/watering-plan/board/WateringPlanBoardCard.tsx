import { Link } from '@tanstack/react-router'
import { Avatar, AvatarFallback, AvatarImage, Badge, KanbanCard } from '@green-ecolution/ui'
import { PlanStatus as WateringPlanStatus } from '@green-ecolution/backend-client'
import type { PlanInListResponse } from '@green-ecolution/backend-client'
import type { User } from '@/api/backendApi'
import { useWateringPlanStatusDetails } from '@/hooks/details/useDetailsForWateringPlanStatus'
import { columnForStatus } from '@/lib/wateringPlanBoard'
import { formatLiters } from '@/lib/utils'
import { capacityDisplay } from '@/lib/capacity'
import type { ModulePlanning } from '@/modules/types'
import type { ReactNode } from 'react'
import { useTranslation } from 'react-i18next'
import { useDateLocale } from '@/lib/i18n/useFormatters'
import { userInitials, formatBoardDate } from './format'
import { Truck, UserRound } from 'lucide-react'

const statusDot: Record<WateringPlanStatus, string> = {
  [WateringPlanStatus.Planned]: 'bg-dark-300',
  [WateringPlanStatus.Active]: 'bg-green-light',
  [WateringPlanStatus.Finished]: 'bg-green-dark-700',
  [WateringPlanStatus.Canceled]: 'bg-red',
  [WateringPlanStatus.NotCompleted]: 'bg-dark-400',
}

interface WateringPlanBoardCardProps {
  plan: PlanInListResponse
  planning: ModulePlanning
  users: User[]
  assignSlot?: ReactNode
  cardState?: 'idle' | 'dragging' | 'ghost'
}

const WateringPlanBoardCard = ({
  plan,
  planning,
  users,
  assignSlot,
  cardState,
}: WateringPlanBoardCardProps) => {
  const column = columnForStatus(plan.status)
  const getWateringPlanStatusDetails = useWateringPlanStatusDetails()
  const statusDetails = getWateringPlanStatusDetails(plan.status)
  const assigned = users.filter((u) => plan.userIds.includes(u.id))
  const { t, i18n } = useTranslation('wateringPlan')
  const dateLocale = useDateLocale()
  const { stopIcon: StopIcon } = planning
  const { icon: DemandIcon, demandLabelKey } = capacityDisplay[planning.capacityKind]

  return (
    <KanbanCard state={cardState}>
      <Link
        to="/plans/$wateringPlanId"
        params={{ wateringPlanId: plan.id.toString() }}
        className="block focus-visible:outline-hidden"
      >
        <div className="flex items-center gap-2">
          <span aria-hidden className={`size-2 shrink-0 rounded-full ${statusDot[plan.status]}`} />
          <p className="font-lato font-semibold text-dark">
            {formatBoardDate(plan.date, dateLocale)}
          </p>
        </div>
        {plan.description && (
          <p className="mt-0.5 line-clamp-1 text-sm text-dark-600">{plan.description}</p>
        )}
        <div className="mt-2.5 flex flex-wrap gap-1.5">
          {column === 'done' && (
            <Badge
              variant={statusDetails.color}
              className="whitespace-nowrap"
              title={plan.cancellationNote || undefined}
            >
              {statusDetails.label}
            </Badge>
          )}
          <Badge variant="muted" className="gap-1 tabular-nums">
            <StopIcon className="size-3" />
            {i18n.t(planning.stopCountKey, { count: plan.stops.length })}
          </Badge>
          <Badge variant="muted" className="gap-1 tabular-nums" title={t(demandLabelKey)}>
            <DemandIcon className="size-3" />
            {formatLiters(plan.totalDemand)}
          </Badge>
          <Badge variant="muted" className="gap-1">
            <Truck className="size-3" />
            {plan.transporter.numberPlate}
            {plan.trailer ? ` | ${plan.trailer.numberPlate}` : ''}
          </Badge>
        </div>
      </Link>
      {(assigned.length > 0 || assignSlot) && (
        <div className="mt-3 flex items-center gap-2 border-t border-dark-100 pt-3">
          {assigned.length > 0 ? (
            <>
              <div className="flex -space-x-1.5">
                {assigned.map((user) => (
                  <Avatar key={user.id} size="xs" className="ring-2 ring-white">
                    {user.avatarUrl && <AvatarImage src={user.avatarUrl} alt="" />}
                    <AvatarFallback variant="user">{userInitials(user)}</AvatarFallback>
                  </Avatar>
                ))}
              </div>
              <p className="truncate text-xs text-dark-600">
                {assigned.map((u) => `${u.firstName.charAt(0)}. ${u.lastName}`).join(', ')}
              </p>
            </>
          ) : (
            assignSlot && (
              <>
                <Avatar size="xs">
                  <AvatarFallback>
                    <UserRound className="size-3 text-dark-500" />
                  </AvatarFallback>
                </Avatar>
                <p className="text-xs text-dark-600">{t('board.assignDriverLabel')}</p>
              </>
            )
          )}
          {assignSlot && <div className="ml-auto">{assignSlot}</div>}
        </div>
      )}
    </KanbanCard>
  )
}

export default WateringPlanBoardCard
