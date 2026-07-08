import { Link } from '@tanstack/react-router'
import { Avatar, AvatarFallback, Badge, KanbanCard } from '@green-ecolution/ui'
import { WateringPlanStatus } from '@green-ecolution/backend-client'
import type { User, WateringPlanInList } from '@/api/backendApi'
import { getWateringPlanStatusDetails } from '@/hooks/details/useDetailsForWateringPlanStatus'
import { columnForStatus } from '@/lib/wateringPlanBoard'
import { formatLiters } from '@/lib/utils'
import type { ReactNode } from 'react'
import { userInitials, formatBoardDate } from './format'

const statusDot: Record<WateringPlanStatus, string> = {
  [WateringPlanStatus.Planned]: 'bg-dark-300',
  [WateringPlanStatus.Active]: 'bg-green-light',
  [WateringPlanStatus.Finished]: 'bg-green-dark',
  [WateringPlanStatus.Canceled]: 'bg-red',
  [WateringPlanStatus.NotCompeted]: 'bg-yellow',
  [WateringPlanStatus.Unknown]: 'bg-dark-300',
}

interface WateringPlanBoardCardProps {
  plan: WateringPlanInList
  users: User[]
  assignSlot?: ReactNode
  cardState?: 'idle' | 'dragging' | 'ghost'
}

const WateringPlanBoardCard = ({
  plan,
  users,
  assignSlot,
  cardState,
}: WateringPlanBoardCardProps) => {
  const column = columnForStatus(plan.status)
  const statusDetails = getWateringPlanStatusDetails(plan.status)
  const assigned = users.filter((u) => plan.userIds.includes(u.id))

  return (
    <KanbanCard state={cardState}>
      <Link
        to="/watering-plans/$wateringPlanId"
        params={{ wateringPlanId: plan.id.toString() }}
        className="block focus-visible:outline-hidden"
      >
        <div className="flex items-center gap-2">
          <span aria-hidden className={`size-2 shrink-0 rounded-full ${statusDot[plan.status]}`} />
          <p className="font-lato font-semibold text-dark">{formatBoardDate(plan.date)}</p>
          {column === 'done' && (
            <Badge
              variant={statusDetails.color}
              className="ml-auto"
              title={plan.cancellationNote || undefined}
            >
              {statusDetails.label}
            </Badge>
          )}
        </div>
        {plan.description && (
          <p className="mt-0.5 line-clamp-1 text-sm text-dark-600">{plan.description}</p>
        )}
        <div className="mt-2.5 flex flex-wrap gap-1.5">
          <Badge variant="muted" className="tabular-nums">
            {plan.treeclusters.length} {plan.treeclusters.length === 1 ? 'Gruppe' : 'Gruppen'}
          </Badge>
          <Badge variant="muted" className="tabular-nums">
            {formatLiters(plan.totalWaterRequired)}
          </Badge>
          <Badge variant="muted">
            {plan.transporter.numberPlate}
            {plan.trailer ? ` | ${plan.trailer.numberPlate}` : ''}
          </Badge>
        </div>
      </Link>
      {(assigned.length > 0 || assignSlot) && (
        <div className="mt-3 flex items-center gap-2 border-t border-dark-100 pt-3">
          {assigned.length > 0 && (
            <>
              <div className="flex -space-x-1.5">
                {assigned.map((user) => (
                  <Avatar key={user.id} size="xs" className="ring-2 ring-white">
                    <AvatarFallback variant="user">{userInitials(user)}</AvatarFallback>
                  </Avatar>
                ))}
              </div>
              <p className="truncate text-xs text-dark-600">
                {assigned.map((u) => `${u.firstName.charAt(0)}. ${u.lastName}`).join(', ')}
              </p>
            </>
          )}
          {assignSlot && <div className="ml-auto">{assignSlot}</div>}
        </div>
      )}
    </KanbanCard>
  )
}

export default WateringPlanBoardCard
