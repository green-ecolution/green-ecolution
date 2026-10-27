import { createFileRoute } from '@tanstack/react-router'
import { Plus } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { z } from 'zod'
import { PlanStatus as WateringPlanStatus } from '@green-ecolution/backend-client'
import { modulesQuery, userQueries, wateringPlanQueries } from '@/api/queries'
import ButtonLink from '@/components/general/links/ButtonLink'
import ListPageHeader from '@/components/general/ListPageHeader'
import PlanModuleTabs from '@/components/watering-plan/PlanModuleTabs'
import WateringPlanBoard from '@/components/watering-plan/board/WateringPlanBoard'
import { pendingLoading, prefetch } from '@/lib/router'
import { activeManifestsFrom } from '@/modules/activeModules'
import ModuleInactive from '@/modules/ModuleInactive'
import { planModule, planTextKey } from '@/modules/planning'
import { moduleManifests } from '@/modules/registry'
import { useActiveModules } from '@/modules/useActiveModules'
import { usePlanPermission } from '@/modules/usePlanPermission'

const boardSearchSchema = z.object({
  module: z.string().optional().catch(undefined),
})

export const Route = createFileRoute('/_protected/plans/')({
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
  const { t, i18n } = useTranslation('wateringPlan')
  const { module } = Route.useSearch()
  const { manifests } = useActiveModules()
  const planManifest = planModule({ module }, manifests)
  const canModify = usePlanPermission(planManifest?.planning, 'update')
  const canCreate = usePlanPermission(planManifest?.planning, 'create')

  if (!planManifest) return <ModuleInactive />

  return (
    <div className="mt-6">
      <div className="container">
        <ListPageHeader
          title={t('list.title')}
          description={i18n.t(
            canModify
              ? planTextKey(
                  planManifest.planning,
                  'listDescriptionCanModify',
                  'wateringPlan:list.descriptionCanModify',
                )
              : planTextKey(
                  planManifest.planning,
                  'listDescription',
                  'wateringPlan:list.description',
                ),
          )}
          action={
            canCreate && (
              <ButtonLink
                icon={Plus}
                label={t('list.createButton')}
                link={{ to: '/plans/new', search: { module: planManifest.key } }}
              />
            )
          }
        />
      </div>
      <section className="px-4 lg:px-10">
        <PlanModuleTabs current={planManifest.key} />
        <WateringPlanBoard module={module} />
      </section>
    </div>
  )
}
