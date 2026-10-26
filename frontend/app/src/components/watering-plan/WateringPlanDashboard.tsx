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
import {
  useWateringPlanStatusDetails,
  showWateringPlanStatusButton,
} from '@/hooks/details/useDetailsForWateringPlanStatus'
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
import WateringPlanPreviewRoute from './WateringPlanRoutePreview'
import { useHasPermission } from '@/lib/auth/useHasPermission'
import { planningFor } from '@/modules/planning'
import { useActiveModules } from '@/modules/useActiveModules'

interface WateringPlanDashboardProps {
  wateringPlan: PlanResponse
}

const WateringPlanDashboard = ({ wateringPlan }: WateringPlanDashboardProps) => {
  const getWateringPlanStatusDetails = useWateringPlanStatusDetails()
  const statusDetails = getWateringPlanStatusDetails(wateringPlan.status)
  const canEdit = useHasPermission(['watering_plan:update'])
  const { t, i18n } = useTranslation(['wateringPlan', 'common'])
  const dateLocale = useDateLocale()
  const { manifests } = useActiveModules()
  const planning = planningFor(manifests, wateringPlan.module)

  const date = wateringPlan?.date
    ? format(new Date(wateringPlan?.date), 'dd.MM.yyyy', { locale: dateLocale })
    : t('common:state.noData')

  const { mutate: downloadGpx } = useDownloadGpx(wateringPlan.gpxUrl)

  return (
    <>
      <EntityDetailHeader
        breakpoint="xl"
        backLink={{ link: { to: '/watering-plans' }, label: t('detail.backLabel') }}
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
                  to: `/watering-plans/$wateringPlanId/edit`,
                  params: { wateringPlanId: String(wateringPlan.id) },
                },
              }
            : undefined
        }
      >
        {wateringPlan.description && <p className="mb-4">{wateringPlan.description}</p>}
        <div className="flex flex-wrap gap-4 items-center">
          {canEdit && showWateringPlanStatusButton(wateringPlan) && (
            <ButtonLink
              link={{
                to: '/watering-plans/$wateringPlanId/status/edit',
                params: { wateringPlanId: wateringPlan.id.toString() },
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
        {wateringPlan.distance == 0 && (
          <Alert variant="destructive" className="mt-6 flex items-center gap-3">
            <AlertIcon variant="destructive" />
            <AlertContent>
              <AlertDescription>{t('detail.routeErrorAlert')}</AlertDescription>
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
          {wateringPlan.distance > 0 && (
            <TabsTrigger value="route">
              <Route className="w-5 h-5" />
              <span className="hidden group-data-[state=active]:block lg:block">
                {t('detail.tabRoute')}
              </span>
            </TabsTrigger>
          )}
        </TabsList>
        <TabsContent value="general">
          <TabGeneralData wateringPlan={wateringPlan} planning={planning} />
        </TabsContent>
        {planning && (
          <TabsContent value="stops">
            <Suspense fallback={null}>
              <planning.StopList plan={wateringPlan} />
            </Suspense>
          </TabsContent>
        )}
        {wateringPlan.distance > 0 && (
          <TabsContent value="route">
            <WateringPlanPreviewRoute wateringPlan={wateringPlan} planning={planning} />
          </TabsContent>
        )}
      </Tabs>

      <div className="mt-10">
        <CommentsSection subject={{ kind: 'plan', id: wateringPlan.id.toString() }} />
      </div>
    </>
  )
}

export default WateringPlanDashboard
