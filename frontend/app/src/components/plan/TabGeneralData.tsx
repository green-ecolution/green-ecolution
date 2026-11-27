import { PlanStatus } from '@green-ecolution/backend-client'
import type { PlanResponse } from '@green-ecolution/backend-client'
import type { ModulePlanning } from '@/modules/types'
import { capacityDisplay } from '@/lib/capacity'
import { Suspense } from 'react'
import { DetailedList, StatusCard } from '@green-ecolution/ui'
import StatusCardGrid from '../general/StatusCardGrid'
import { format, formatDuration, intervalToDuration } from 'date-fns'
import { useTranslation } from 'react-i18next'
import { useSuspenseQuery, useQuery } from '@tanstack/react-query'
import { userQueries, routingStartPointsQuery } from '@/api/queries'
import { useDateLocale } from '@/lib/i18n/useFormatters'
import { formatKm } from '@/lib/utils'

interface TabGeneralDataProps {
  plan: PlanResponse
  /** Absent when the plan's module is not active; its rows and summary are left out then. */
  planning: ModulePlanning | undefined
}

const TabGeneralData: React.FC<TabGeneralDataProps> = ({ plan, planning }) => {
  const { t, i18n } = useTranslation(['wateringPlan', 'common'])
  const dateLocale = useDateLocale()
  const { data: userRes } = useSuspenseQuery(userQueries.list({ page: 1, perPage: 100 }))
  const { data: startPoints } = useQuery(routingStartPointsQuery())
  const defaultStartPointName = startPoints?.[0]?.name

  const assignedUsers = userRes.data?.filter((user) => plan.userIds.includes(user.id)) ?? []
  const noData = t('common:state.noData')
  const capacity = capacityDisplay[planning?.capacityKind ?? 'water']

  const updatedDate = plan?.updatedAt
    ? format(new Date(plan.updatedAt), 'dd.MM.yyyy', { locale: dateLocale })
    : noData

  const planData: {
    label: string
    value: string
  }[] = [
    ...(planning
      ? [
          {
            label: t(capacityDisplay[planning.capacityKind].demandLabelKey),
            value: plan.totalDemand ? t('detail.demandValue', { value: plan.totalDemand }) : noData,
          },
        ]
      : []),
    {
      label: t('detail.transporterLabel'),
      value: plan.transporter
        ? plan.transporter.archivedAt
          ? t('detail.archivedVehicleValue', { numberPlate: plan.transporter.numberPlate })
          : plan.transporter.numberPlate
        : noData,
    },
    {
      label: t('detail.trailerLabel'),
      value: plan.trailer
        ? plan.trailer.archivedAt
          ? t('detail.archivedVehicleValue', { numberPlate: plan.trailer.numberPlate })
          : plan.trailer.numberPlate
        : noData,
    },
    ...(plan.trailer
      ? [
          {
            label: t(capacity.transporterLabelKey),
            value: plan.transporterCapacityUsed
              ? t(capacity.transporterUsedValueKey)
              : t(capacity.transporterUnusedValueKey),
          },
        ]
      : []),
    ...(planning
      ? [
          {
            label: i18n.t(planning.stopCountLabelKey),
            value: plan.stops.length
              ? i18n.t(planning.stopCountKey, { count: plan.stops.length })
              : noData,
          },
        ]
      : []),
    {
      label: t('detail.assignedUsersLabel'),
      value: assignedUsers.length
        ? assignedUsers.map((user) => `${user.firstName} ${user.lastName}`).join(', ')
        : noData,
    },
    {
      label: t('detail.refillCountLabel'),
      value: plan.refillCount ? plan.refillCount.toString() : noData,
    },
    {
      label: t('detail.durationLabel'),
      value: plan.duration
        ? `${formatDuration(intervalToDuration({ start: 0, end: plan.duration * 1000 }), { format: ['hours', 'minutes'], delimiter: ', ', locale: dateLocale })}`
        : noData,
    },
    {
      label: t('detail.updatedAtLabel'),
      value: updatedDate,
    },
  ]

  return (
    <>
      <StatusCardGrid>
        {plan?.status === PlanStatus.Canceled && plan.cancellationNote && (
          <li>
            <StatusCard
              label={t('detail.cancellationNoteLabel')}
              value=""
              description={plan.cancellationNote}
            />
          </li>
        )}
        {planning && (
          <Suspense fallback={null}>
            <planning.PlanSummary plan={plan} />
          </Suspense>
        )}
        <li>
          <StatusCard
            label={t('detail.routeLengthLabel')}
            value={formatKm(plan.distance)}
            isLarge
            description={
              (plan.startPointName ?? defaultStartPointName)
                ? t('detail.routeStartDescription', {
                    startPoint: plan.startPointName ?? defaultStartPointName,
                  })
                : undefined
            }
          />
        </li>
      </StatusCardGrid>

      <section className="mt-16">
        <DetailedList headline={t('detail.dataHeadline')} details={planData} />
      </section>
    </>
  )
}

export default TabGeneralData
