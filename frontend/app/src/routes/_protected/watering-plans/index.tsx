import { createFileRoute } from '@tanstack/react-router'
import { Plus } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { WateringPlanStatus } from '@green-ecolution/backend-client'
import { clusterQueries, userQueries, wateringPlanQueries } from '@/api/queries'
import ButtonLink from '@/components/general/links/ButtonLink'
import ListPageHeader from '@/components/general/ListPageHeader'
import WateringPlanBoard from '@/components/watering-plan/board/WateringPlanBoard'
import { pendingLoading, prefetch } from '@/lib/router'
import { Can } from '@/lib/auth/Can'
import { useHasPermission } from '@/lib/auth/useHasPermission'

export const Route = createFileRoute('/_protected/watering-plans/')({
  component: WateringPlans,
  pendingComponent: pendingLoading({ key: 'wateringPlan:list.loadingLabel' }),
  loader: ({ context: { queryClient } }) => {
    prefetch(
      queryClient,
      wateringPlanQueries.boardColumn([WateringPlanStatus.Planned]),
      'wateringPlanQueries.boardColumn(planned)',
    )
    prefetch(
      queryClient,
      wateringPlanQueries.boardColumn([WateringPlanStatus.Active]),
      'wateringPlanQueries.boardColumn(active)',
    )
    prefetch(queryClient, clusterQueries.suggested(), 'clusterQueries.suggested')
    prefetch(queryClient, userQueries.list({ page: 1, perPage: 100 }), 'userQueries.list')
  },
})

function WateringPlans() {
  const { t } = useTranslation('wateringPlan')
  const canModify = useHasPermission(['watering_plan:update'])

  return (
    <div className="mt-6">
      <div className="container">
        <ListPageHeader
          title={t('list.title')}
          description={canModify ? t('list.descriptionCanModify') : t('list.description')}
          action={
            <Can permission={['watering_plan:create']}>
              <ButtonLink
                icon={Plus}
                label={t('list.createButton')}
                link={{ to: '/watering-plans/new' }}
              />
            </Can>
          }
        />
      </div>
      <section className="px-4 lg:px-10">
        <WateringPlanBoard />
      </section>
    </div>
  )
}
