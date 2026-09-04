import { WateringPlanStatus } from '@green-ecolution/backend-client'
import type { WateringPlan } from '@/api/backendApi'
import { DetailedList, StatusCard } from '@green-ecolution/ui'
import StatusCardGrid from '../general/StatusCardGrid'
import { format, formatDuration, intervalToDuration } from 'date-fns'
import { useTranslation } from 'react-i18next'
import { useWateringPlanStatusDetails } from '@/hooks/details/useDetailsForWateringPlanStatus'
import { useSuspenseQuery, useQuery } from '@tanstack/react-query'
import { userQueries, routingStartPointsQuery } from '@/api/queries'
import { useDateLocale } from '@/lib/i18n/useFormatters'
import { formatKm } from '@/lib/utils'

interface TabGeneralDataProps {
  wateringPlan: WateringPlan
}

const TabGeneralData: React.FC<TabGeneralDataProps> = ({ wateringPlan }) => {
  const { t } = useTranslation(['wateringPlan', 'common'])
  const dateLocale = useDateLocale()
  const getWateringPlanStatusDetails = useWateringPlanStatusDetails()
  const { data: userRes } = useSuspenseQuery(userQueries.list({ page: 1, perPage: 100 }))
  const { data: startPoints } = useQuery(routingStartPointsQuery())
  const defaultStartPointName = startPoints?.[0]?.name

  const assignedUsers = userRes.data?.filter((user) => wateringPlan.userIds.includes(user.id)) ?? []
  const noData = t('common:state.noData')

  const updatedDate = wateringPlan?.updatedAt
    ? format(new Date(wateringPlan.updatedAt), 'dd.MM.yyyy', { locale: dateLocale })
    : noData

  const wateringPlanData: {
    label: string
    value: string
  }[] = [
    {
      label: t('detail.routeLengthLabel'),
      value: wateringPlan.distance ? formatKm(wateringPlan.distance) : noData,
    },
    {
      label: t('detail.startPointLabel'),
      value: wateringPlan.startPointName ?? defaultStartPointName ?? noData,
    },
    {
      label: t('detail.requiredWaterLabel'),
      value: wateringPlan.totalWaterRequired
        ? t('detail.requiredWaterValue', { value: wateringPlan.totalWaterRequired })
        : noData,
    },
    {
      label: t('detail.transporterLabel'),
      value: wateringPlan.transporter
        ? wateringPlan.transporter.archivedAt
          ? t('detail.archivedVehicleValue', { numberPlate: wateringPlan.transporter.numberPlate })
          : wateringPlan.transporter.numberPlate
        : noData,
    },
    {
      label: t('detail.trailerLabel'),
      value: wateringPlan.trailer
        ? wateringPlan.trailer.archivedAt
          ? t('detail.archivedVehicleValue', { numberPlate: wateringPlan.trailer.numberPlate })
          : wateringPlan.trailer.numberPlate
        : noData,
    },
    {
      label: t('detail.clusterCountLabel'),
      value: wateringPlan.treeclusters?.length
        ? t('detail.clusterCountValue', { count: wateringPlan.treeclusters.length })
        : noData,
    },
    {
      label: t('detail.assignedUsersLabel'),
      value: assignedUsers.length
        ? assignedUsers.map((user) => `${user.firstName} ${user.lastName}`).join(', ')
        : noData,
    },
    {
      label: t('detail.refillCountLabel'),
      value: wateringPlan.refillCount ? wateringPlan.refillCount.toString() : noData,
    },
    {
      label: t('detail.durationLabel'),
      value: wateringPlan.duration
        ? `${formatDuration(intervalToDuration({ start: 0, end: wateringPlan.duration * 1000 }), { format: ['hours', 'minutes'], delimiter: ', ', locale: dateLocale })}`
        : noData,
    },
    {
      label: t('detail.updatedAtLabel'),
      value: updatedDate,
    },
  ]

  const statusDetails = getWateringPlanStatusDetails(wateringPlan.status)

  return (
    <>
      <StatusCardGrid>
        <li>
          <StatusCard
            status={statusDetails.color}
            indicator="badge"
            label={t('detail.currentStatusLabel')}
            value={statusDetails.label}
            description={statusDetails.description}
          />
        </li>
        {wateringPlan?.status === WateringPlanStatus.Canceled && wateringPlan.cancellationNote && (
          <li>
            <StatusCard
              label={t('detail.cancellationNoteLabel')}
              value=""
              description={wateringPlan.cancellationNote}
            />
          </li>
        )}
        {wateringPlan?.status === WateringPlanStatus.Finished && (
          <li>
            <StatusCard
              label={t('detail.consumedWaterLabel')}
              value={t('detail.consumedWaterValue', {
                value: wateringPlan.evaluation.reduce(
                  (sum: number, item: { consumedWater: number }) => sum + item.consumedWater,
                  0,
                ),
              })}
              isLarge
              description={t('detail.consumedWaterDescription', {
                count: wateringPlan.treeclusters.length,
              })}
            />
          </li>
        )}
        <li>
          <StatusCard
            label={t('detail.routeLengthLabel')}
            value={formatKm(wateringPlan.distance)}
            isLarge
            description={
              (wateringPlan.startPointName ?? defaultStartPointName)
                ? t('detail.routeStartDescription', {
                    startPoint: wateringPlan.startPointName ?? defaultStartPointName,
                  })
                : undefined
            }
          />
        </li>
      </StatusCardGrid>

      <section className="mt-16">
        <DetailedList headline={t('detail.dataHeadline')} details={wateringPlanData} />
      </section>
    </>
  )
}

export default TabGeneralData
