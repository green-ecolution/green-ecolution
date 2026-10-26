import { createFileRoute } from '@tanstack/react-router'
import { Plus } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { z } from 'zod'
import { PlanStatus as WateringPlanStatus } from '@green-ecolution/backend-client'
import { modulesQuery, userQueries, wateringPlanQueries } from '@/api/queries'
import ButtonLink from '@/components/general/links/ButtonLink'
import ListPageHeader from '@/components/general/ListPageHeader'
import WateringPlanBoard from '@/components/watering-plan/board/WateringPlanBoard'
import { pendingLoading, prefetch } from '@/lib/router'
import { Can } from '@/lib/auth/Can'
import { useHasPermission } from '@/lib/auth/useHasPermission'
import { activeManifestsFrom } from '@/modules/activeModules'
import ModuleInactive from '@/modules/ModuleInactive'
import { planModule } from '@/modules/planning'
import { moduleManifests } from '@/modules/registry'
import { useActiveModules } from '@/modules/useActiveModules'

const boardSearchSchema = z.object({
  module: z.string().optional().catch(undefined),
})

export const Route = createFileRoute('/_protected/watering-plans/')({
  component: WateringPlans,
  validateSearch: boardSearchSchema,
  pendingComponent: pendingLoading({ key: 'wateringPlan:list.loadingLabel' }),
  loaderDeps: ({ search: { module } }) => ({ module }),
  loader: async ({ context: { queryClient }, deps }) => {
    const modules = await queryClient.ensureQueryData(modulesQuery())
    const planManifest = planModule(deps, activeManifestsFrom(moduleManifests, modules))
    if (!planManifest) return
    planManifest.planning.prefetch?.(queryClient)
    prefetch(
      queryClient,
      wateringPlanQueries.boardColumn(planManifest.key, [WateringPlanStatus.Planned]),
      'wateringPlanQueries.boardColumn(planned)',
    )
    prefetch(
      queryClient,
      wateringPlanQueries.boardColumn(planManifest.key, [WateringPlanStatus.Active]),
      'wateringPlanQueries.boardColumn(active)',
    )
    prefetch(queryClient, userQueries.list({ page: 1, perPage: 100 }), 'userQueries.list')
  },
})

function WateringPlans() {
  const { t } = useTranslation('wateringPlan')
  const canModify = useHasPermission(['watering_plan:update'])
  const { module } = Route.useSearch()
  const { manifests } = useActiveModules()

  if (!planModule({ module }, manifests)) return <ModuleInactive />

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
        <WateringPlanBoard module={module} />
      </section>
    </div>
  )
}
