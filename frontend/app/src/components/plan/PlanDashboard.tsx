import EntityDetailHeader from '../general/EntityDetailHeader'
import {
  Badge,
  Button,
  Tabs,
  TabsList,
  TabsTrigger,
  TabsContent,
  Alert,
  AlertIcon,
  AlertContent,
  AlertDescription,
} from '@green-ecolution/ui'
import { usePlanStatusDetails, showPlanStatusButton } from '@/hooks/details/useDetailsForPlanStatus'
import { format } from 'date-fns'
import { File, MoveRight, Route } from 'lucide-react'
import { Suspense } from 'react'
import { useTranslation } from 'react-i18next'
import TabGeneralData from './TabGeneralData'
import CommentsSection from '@/components/comments/CommentsSection'
import ButtonLink from '../general/links/ButtonLink'
import type { PlanResponse } from '@green-ecolution/backend-client'
import { useDownloadGpx } from '@/hooks/useDownloadGpx'
import { useDateLocale } from '@/lib/i18n/useFormatters'
import PlanPreviewRoute from './PlanRoutePreview'
import { usePlanPermission } from '@/modules/usePlanPermission'
import { planningFor, planTextKey } from '@/modules/planning'
import { useActiveModules } from '@/modules/useActiveModules'

interface PlanDashboardProps {
  plan: PlanResponse
}

const PlanDashboard = ({ plan: plan }: PlanDashboardProps) => {
  const getPlanStatusDetails = usePlanStatusDetails()
  const statusDetails = getPlanStatusDetails(plan.status)
  const { t, i18n } = useTranslation(['wateringPlan', 'common'])
  const dateLocale = useDateLocale()
  const { manifests } = useActiveModules()
  const planning = planningFor(manifests, plan.module)
  const canEdit = usePlanPermission(planning, 'update')

  const date = plan?.date
    ? format(new Date(plan?.date), 'dd.MM.yyyy', { locale: dateLocale })
    : t('common:state.noData')

  const { mutate: downloadGpx } = useDownloadGpx(plan.gpxUrl)

  return (
    <>
      <EntityDetailHeader
        breakpoint="xl"
        backLink={{ link: { to: '/plans' }, label: t('detail.backLabel') }}
        title={<>{t('detail.title', { date })}</>}
        badge={
          <Badge variant={statusDetails?.color ?? 'outline-dark'} size="lg">
            {statusDetails?.label ?? t('common:state.noData')}
          </Badge>
        }
        editLink={
          canEdit
            ? {
                label: t('detail.editLabel'),
                link: {
                  to: `/plans/$wateringPlanId/edit`,
                  params: { wateringPlanId: String(plan.id) },
                },
              }
            : undefined
        }
      >
        <p className="mb-4 text-dark-600">{statusDetails.description}</p>
        {plan.description && <p className="mb-4">{plan.description}</p>}
        <div className="flex flex-wrap gap-4 items-center">
          {canEdit && showPlanStatusButton(plan) && (
            <ButtonLink
              link={{
                to: '/plans/$wateringPlanId/status/edit',
                params: { wateringPlanId: plan.id.toString() },
              }}
              label={t('detail.updateStatusLabel')}
              icon={MoveRight}
            />
          )}
          <Button variant="nav" onClick={() => downloadGpx()} className="p-0 h-auto [&_svg]:size-4">
            {t('detail.downloadRouteLabel')}
            <MoveRight className="icon-arrow-animate" />
          </Button>
        </div>
        {plan.distance == 0 && (
          <Alert variant="destructive" className="mt-6 flex items-center gap-3">
            <AlertIcon variant="destructive" />
            <AlertContent>
              <AlertDescription>
                {i18n.t(
                  planTextKey(planning, 'routeErrorAlert', 'wateringPlan:detail.routeErrorAlert'),
                )}
              </AlertDescription>
            </AlertContent>
          </Alert>
        )}
      </EntityDetailHeader>

      <Tabs defaultValue="general" className="mt-10">
        <TabsList>
          <TabsTrigger value="general">
            <File className="w-5 h-5" />
            <span className="hidden group-data-[state=active]:block lg:block">
              {t('detail.tabGeneral')}
            </span>
          </TabsTrigger>
          {planning && (
            <TabsTrigger value="stops">
              <planning.stopIcon className="w-5 h-5" />
              <span className="hidden group-data-[state=active]:block lg:block">
                {i18n.t(planning.stopsLabelKey)}
              </span>
            </TabsTrigger>
          )}
          {plan.distance > 0 && (
            <TabsTrigger value="route">
              <Route className="w-5 h-5" />
              <span className="hidden group-data-[state=active]:block lg:block">
                {t('detail.tabRoute')}
              </span>
            </TabsTrigger>
          )}
        </TabsList>
        <TabsContent value="general">
          <TabGeneralData plan={plan} planning={planning} />
        </TabsContent>
        {planning && (
          <TabsContent value="stops">
            <Suspense fallback={null}>
              <planning.StopList plan={plan} />
            </Suspense>
          </TabsContent>
        )}
        {plan.distance > 0 && (
          <TabsContent value="route">
            <PlanPreviewRoute plan={plan} planning={planning} />
          </TabsContent>
        )}
      </Tabs>

      <div className="mt-10">
        <CommentsSection subject={{ kind: 'plan', id: plan.id.toString() }} />
      </div>
    </>
  )
}

export default PlanDashboard
